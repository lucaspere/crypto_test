use chrono::Utc;
use futures::StreamExt;
use rayon::{
    iter::{IntoParallelIterator, ParallelIterator},
    slice::ParallelSlice,
};
use rust_decimal::Decimal;
use std::{collections::HashMap, sync::Arc, time::Instant};
use tokio::sync::Semaphore;
use tracing::{debug, info, instrument, warn, Span};

use crate::{
    apis::api_models::request::TokenValueDataRequest,
    container::ServiceContainer,
    models::{
        token_picks::{TokenPick, TokenPickResponse},
        tokens::Token,
    },
    utils::{errors::app_error::AppError, redis_keys::RedisKeys, time::TimePeriod},
};

const PROCESSING_LOCK_TTL: u64 = 180; // 3 minutes
const BATCH_SIZE: i64 = 50;
const DB_SLOW_THRESHOLD_SECS: f64 = 2.0;

#[instrument(skip(app_state), fields(job_id = %uuid::Uuid::new_v4()))]
pub async fn process_token_picks_job(app_state: &Arc<ServiceContainer>) -> Result<(), AppError> {
    let processing_lock_key = format!("{}-processing-lock", app_state.environment);
    let lock_acquired = app_state
        .redis_service
        .set_nx(&processing_lock_key, "1", PROCESSING_LOCK_TTL)
        .await
        .map_err(|e| {
            warn!("Failed to acquire Redis lock: {}", e);
            AppError::RedisError(e)
        })?;

    if !lock_acquired {
        debug!("Another instance is currently processing token picks");
        return Ok(());
    }

    let start_time = Instant::now();
    info!("Starting token picks processing");

    let result = async {
        let since = if app_state.environment == "staging" {
            Utc::now() - chrono::Duration::days(2)
        } else {
            Utc::now() - chrono::Duration::days(30)
        };
        let tokens = app_state.token_service.get_all_tokens(since).await?;

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

                let _permit = semaphore.acquire().await.map_err(|_| {
                    warn!("Failed to acquire semaphore");
                    AppError::InternalServerError()
                })?;

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

        Ok(())
    }
    .await;

    let duration = start_time.elapsed();
    info!(
        duration_secs = duration.as_secs_f64(),
        success = result.is_ok(),
        "Finished processing token picks"
    );

    // Release the lock early on error
    if result.is_err() {
        if let Err(e) = app_state
            .redis_service
            .delete_cached(&processing_lock_key)
            .await
        {
            debug!(error = ?e, "Failed to release processing lock after error");
        }
    }

    result
}

#[instrument(skip(app_state, tokens, address_batch), fields(batch_size = address_batch.len()))]
async fn process_address_batch(
    app_state: &Arc<ServiceContainer>,
    address_batch: &[String],
    tokens: &HashMap<String, Vec<TokenPick>>,
) -> Result<(), AppError> {
    let start = Instant::now();

    let latest_token_info = {
        let api_start = Instant::now();
        let result = app_state
            .token_service
            .get_token_value_data(TokenValueDataRequest {
                address: address_batch.to_vec(),
                time_period: Some(TimePeriod::Day),
            })
            .await?;

        let duration = api_start.elapsed().as_secs_f64();
        debug!(duration = duration, "Retrieved latest token metadata");
        result
    };

    let tokens_to_save = address_batch
        .iter()
        .filter_map(|address| latest_token_info.get(address))
        .map(|metadata| Token::from(metadata.clone()))
        .collect::<Vec<_>>();

    app_state
        .token_service
        .save_many_tokens(tokens_to_save)
        .await?;

    let processing_futures = latest_token_info.into_iter().map(|(address, metadata)| {
        let picks = tokens.get(&address).unwrap();
        let supply = picks.iter().find_map(|pick| pick.supply_at_call).unwrap();
        process_token_picks(
            app_state,
            picks,
            metadata.price,
            supply,
            metadata.liquidity,
            metadata.volume,
        )
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
    picks: &Vec<TokenPick>,
    price: Decimal,
    supply: Decimal,
    liquidity: Decimal,
    volume_24h: Decimal,
) -> Result<Vec<TokenPickResponse>, AppError> {
    let mut picks = picks.clone();
    let pick_futures = picks.iter_mut().map(|pick| {
        app_state
            .token_service
            .process_single_pick_v2(pick, price, supply, liquidity, volume_24h)
    });

    let results = futures::future::join_all(pick_futures).await;

    let cache_futures = results.iter().filter_map(|r| {
        let pick = r.as_ref().unwrap().0.clone();
        if TokenPick::is_qualified(
            pick.market_cap_at_call,
            pick.token.liquidity,
            pick.token.volume_24h,
        ) {
            Some(update_pick_stats(app_state, pick))
        } else {
            None
        }
    });

    futures::future::join_all(cache_futures).await;

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
) -> Result<(), AppError> {
    let timeframes = [
        (TimePeriod::SixHours, chrono::Duration::hours(6)),
        (TimePeriod::Day, chrono::Duration::days(1)),
        (TimePeriod::Week, chrono::Duration::days(7)),
        (TimePeriod::Month, chrono::Duration::days(30)),
        (TimePeriod::AllTime, chrono::Duration::days(365)),
    ];

    let mut pipe = redis::pipe();
    pipe.atomic();

    // Batch all Redis operations into a single atomic pipeline
    for (timeframe, duration) in timeframes.iter() {
        if pick.call_date > (Utc::now() - *duration) {
            let zset_key =
                RedisKeys::get_group_leaderboard_key(pick.group.id, &timeframe.to_string());
            let hash_key =
                RedisKeys::get_group_leaderboard_data_key(pick.group.id, &timeframe.to_string());

            pipe.zadd(&zset_key, pick.id.to_string(), pick.highest_mult_post_call);

            if let Ok(json_data) = serde_json::to_string(&pick) {
                pipe.hset(&hash_key, pick.id.to_string(), json_data);
            }

            let ttl = RedisKeys::get_ttl_for_timeframe(timeframe);

            pipe.expire(&zset_key, ttl);
            pipe.expire(&hash_key, ttl);
        }
    }

    app_state.redis_service.execute_pipe(pipe).await?;
    Ok(())
}
