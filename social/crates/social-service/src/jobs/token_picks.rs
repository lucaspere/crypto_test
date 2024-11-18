use chrono::Utc;
use futures::StreamExt;
use rayon::{
    iter::{IntoParallelIterator, ParallelIterator},
    slice::ParallelSlice,
};
use rust_decimal::prelude::ToPrimitive;
use std::{collections::HashMap, sync::Arc, time::Instant};
use tokio::sync::Semaphore;
use tracing::{debug, info, instrument, Span};

use crate::{
    container::ServiceContainer,
    external_services::rust_monorepo::get_latest_w_metadata::LatestTokenMetadataResponse,
    models::{token_picks::TokenPickResponse, tokens::Token},
    repositories::token_repository::TokenPickRow,
    utils::{api_errors::ApiError, redis_keys::RedisKeys},
};

const PROCESSING_LOCK_KEY: &str = "token_picks:processing_lock";
const PROCESSING_LOCK_TTL: u64 = 300; // 5 minutes
const BATCH_SIZE: i64 = 30;
const DB_SLOW_THRESHOLD_SECS: f64 = 2.0;

#[instrument(skip(app_state), fields(job_id = %uuid::Uuid::new_v4()))]
pub async fn process_token_picks_job(app_state: &Arc<ServiceContainer>) -> Result<(), ApiError> {
    let start_time = Instant::now();
    info!("Starting token picks processing");

    let tokens = app_state.token_service.get_all_tokens().await?;

    let addresses: Vec<_> = tokens.keys().cloned().collect();
    info!(token_count = addresses.len(), "Retrieved tokens to process");

    let semaphore = Arc::new(Semaphore::new(4));
    let chunks: Vec<_> = addresses
        .par_chunks(BATCH_SIZE as usize)
        .map(|c| c.to_vec())
        .collect();

    debug!(chunk_count = chunks.len(), "Created processing chunks");

    let futures = chunks.into_iter().enumerate().map(|(idx, address_batch)| {
        let app_state = Arc::clone(&app_state);
        let semaphore = Arc::clone(&semaphore);
        let tokens = tokens.clone();
        let span = Span::current();

        async move {
            let _guard = span.enter();
            debug!(
                chunk_idx = idx,
                size = address_batch.len(),
                "Processing chunk"
            );

            let _permit = semaphore
                .acquire()
                .await
                .map_err(|_| ApiError::InternalError("Failed to acquire semaphore".to_string()))?;

            let chunk_start = Instant::now();
            let result = process_address_batch(&app_state, &address_batch, &tokens).await;

            let duration = chunk_start.elapsed().as_secs_f64();
            debug!(
                chunk_idx = idx,
                duration = duration,
                "Finished processing chunk"
            );

            result
        }
    });

    futures::stream::iter(futures)
        .buffer_unordered(4)
        .collect::<Vec<_>>()
        .await;

    let duration = start_time.elapsed();
    info!(
        duration_secs = duration.as_secs_f64(),
        "Finished processing token picks"
    );

    Ok(())
}

#[instrument(skip(app_state, tokens, address_batch), fields(batch_size = address_batch.len()))]
async fn process_address_batch(
    app_state: &Arc<ServiceContainer>,
    address_batch: &[String],
    tokens: &HashMap<String, Vec<TokenPickRow>>,
) -> Result<(), ApiError> {
    let start = Instant::now();

    let latest_token_info = {
        let api_start = Instant::now();
        let result = app_state
            .rust_monorepo_service
            .get_latest_w_metadata(&address_batch)
            .await?;

        let duration = api_start.elapsed().as_secs_f64();
        debug!(duration = duration, "Retrieved latest token metadata");
        result
    };

    let tokens_to_save = address_batch
        .iter()
        .map(|address| latest_token_info.get(address).unwrap())
        .map(|metadata| Token::from(metadata.clone()))
        .collect::<Vec<_>>();

    app_state
        .token_service
        .save_many_tokens(tokens_to_save)
        .await?;

    let processing_futures = latest_token_info.into_iter().map(|(address, metadata)| {
        let token = tokens.get(&address).unwrap();
        process_token_picks(app_state, token, metadata)
    });

    let results: Vec<_> = futures::future::join_all(processing_futures)
        .await
        .into_par_iter()
        .filter_map(|r| r.ok())
        .flatten()
        .collect();

    if !results.is_empty() {
        debug!(picks_count = results.len(), "Updating database");
        app_state
            .token_service
            .bulk_update_token_picks(&results)
            .await?;
    }
    let duration = start.elapsed().as_secs_f64();
    debug!(duration = duration, "Completed batch processing");

    Ok(())
}

async fn process_token_picks(
    app_state: &Arc<ServiceContainer>,
    picks: &Vec<TokenPickRow>,
    metadata: LatestTokenMetadataResponse,
) -> Result<Vec<TokenPickResponse>, ApiError> {
    let mut picks = picks.clone();
    let pick_futures = picks.iter_mut().map(|pick| {
        app_state
            .token_service
            .process_single_pick_with_metadata(pick, &metadata)
    });

    let results = futures::future::join_all(pick_futures).await;

    // Update cache and leaderboards concurrently
    let cache_futures = results.iter().filter(|r| r.is_ok()).map(|r| {
        let pick = r.as_ref().unwrap().0.clone();
        update_pick_stats(app_state, pick)
    });

    futures::future::join_all(cache_futures).await;

    // Bulk update database
    let updated_picks: Vec<_> = results
        .into_iter()
        .filter(|r| r.is_ok())
        .map(|r| r.unwrap().0)
        .collect();

    Ok(updated_picks)
}

async fn update_pick_stats(
    app_state: &ServiceContainer,
    pick: TokenPickResponse,
) -> Result<(), ApiError> {
    let timeframes = [
        (RedisKeys::LEADERBOARD_24H, chrono::Duration::hours(24)),
        (RedisKeys::LEADERBOARD_7D, chrono::Duration::days(7)),
        (RedisKeys::LEADERBOARD_1Y, chrono::Duration::days(365)),
    ];

    let futures = timeframes.iter().map(|(timeframe, duration)| {
        let pick = pick.clone();
        let app_state = app_state;

        async move {
            if pick.call_date > (Utc::now() - *duration) {
                update_timeframe_leaderboard(&app_state, &pick, timeframe).await?;
            }
            Ok::<_, ApiError>(())
        }
    });

    futures::future::join_all(futures).await;
    Ok(())
}

async fn update_timeframe_leaderboard(
    app_state: &ServiceContainer,
    pick: &TokenPickResponse,
    timeframe: &str,
) -> Result<(), ApiError> {
    let returns_key = RedisKeys::get_leaderboard_key(timeframe, RedisKeys::METRIC_RETURNS);
    let hit_rate_key = RedisKeys::get_leaderboard_key(timeframe, RedisKeys::METRIC_HIT_RATE);
    let total_picks_key = RedisKeys::get_leaderboard_key(timeframe, RedisKeys::METRIC_TOTAL_PICKS);

    let mut pipe = redis::pipe();

    // Update leaderboards
    pipe.zadd(
        &returns_key,
        &pick.token.address,
        pick.highest_mult_post_call.to_f64().unwrap_or_default(),
    );

    if pick.hit_date.is_some() {
        pipe.zincr(&hit_rate_key, &pick.token.address, 1.0);
    }

    pipe.zincr(&total_picks_key, &pick.token.address, 1.0);

    app_state.redis_service.execute_pipe(pipe).await?;
    Ok(())
}
