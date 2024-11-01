use std::{collections::HashMap, sync::Arc};

use chrono::Utc;
use futures::future::join_all;
use rust_decimal::Decimal;
use sqlx::types::Json;
use tracing::{debug, error, info};

use crate::{
    apis::token_handlers::TokenQuery,
    external_services::{birdeye::BirdeyeService, rust_monorepo::RustMonorepoService},
    models::{
        token_picks::{TokenPick, TokenPickResponse},
        tokens::{Token, TokenPickRequest},
        user_stats::BestPick,
    },
    repositories::token_repository::{ListTokenPicksParams, TokenRepository},
    services::user_service::UserService,
    utils::api_errors::ApiError,
};

use super::redis_service::RedisService;

pub struct TokenService {
    token_repository: Arc<TokenRepository>,
    rust_monorepo_service: Arc<RustMonorepoService>,
    user_service: Arc<UserService>,
    redis_service: Arc<RedisService>,
    birdeye_service: Arc<BirdeyeService>,
}

impl TokenService {
    pub fn new(
        token_repository: Arc<TokenRepository>,
        rust_monorepo_service: Arc<RustMonorepoService>,
        user_service: Arc<UserService>,
        redis_service: Arc<RedisService>,
        birdeye_service: Arc<BirdeyeService>,
    ) -> Self {
        Self {
            token_repository,
            rust_monorepo_service,
            user_service,
            redis_service,
            birdeye_service,
        }
    }

    pub async fn list_token_picks(
        &self,
        query: TokenQuery,
    ) -> Result<Vec<TokenPickResponse>, ApiError> {
        let user = if let Some(username) = query.username {
            self.user_service.get_user_by_username(&username).await?
        } else {
            None
        };

        let params = ListTokenPicksParams {
            user_id: user.map(|u| u.id),
        };

        let cache_key = format!("token_picks:{:?}", params);
        debug!("Checking cache for key: {}", cache_key);

        if let Ok(Some(cached)) = self
            .redis_service
            .get_cached::<Vec<TokenPickResponse>>(&cache_key)
            .await
        {
            info!("Cache hit for token picks");
            return Ok(cached);
        }

        debug!("Listing token picks with params: {:?}", params);

        let mut picks = self
            .token_repository
            .list_token_picks(Some(&params))
            .await
            .map_err(ApiError::from)?;

        let token_addresses: Vec<_> = picks.iter().map(|p| p.token.address.clone()).collect();
        let latest_prices_future = self
            .rust_monorepo_service
            .get_latest_w_metadata(token_addresses);

        let ohlcv_futures = picks.iter().map(|pick| {
            let birdeye_service = self.birdeye_service.clone();
            async move {
                birdeye_service
                    .get_ohlcv_request(
                        &pick.token.chain,
                        &pick.token.address,
                        pick.call_date.timestamp(),
                        Utc::now().timestamp(), // We'll update this with actual price timestamp
                        "15m",
                    )
                    .await
                    .map(|result| (pick.token.address.clone(), result))
            }
        });

        // Execute price and OHLCV requests concurrently
        let (latest_prices, ohlcv_results) =
            tokio::join!(latest_prices_future, join_all(ohlcv_futures));
        let latest_prices = latest_prices?;
        let ohlcv_map: HashMap<_, _> = ohlcv_results.into_iter().filter_map(Result::ok).collect();
        let mut picks_to_update = HashMap::with_capacity(picks.len());
        let mut pick_responses = Vec::with_capacity(picks.len());

        for pick in &mut picks {
            let latest_price = latest_prices
                .get(&pick.token.address)
                .ok_or_else(|| ApiError::InternalServerError("Price data not found".to_string()))?;

            let highest_price = ohlcv_map
                .get(&pick.token.address)
                .ok_or_else(|| ApiError::InternalServerError("OHLCV data not found".to_string()))?;

            let supply = latest_price.metadata.supply.unwrap_or_default();
            let highest_market_cap = highest_price.high * supply;

            if highest_market_cap > pick.highest_market_cap.unwrap_or_default() {
                info!(
                    "New highest market cap for token {}: {}",
                    pick.token.symbol, highest_market_cap
                );
                pick.highest_market_cap = Some(highest_market_cap);
                picks_to_update.insert(pick.id, pick.clone());
            }

            // Create pick response and update stats
            let mut pick_response = TokenPickResponse::from(pick.clone());
            pick_response.current_market_cap =
                latest_price.metadata.mc.unwrap_or_default().round_dp(2);
            pick_response.current_multiplier = calculate_return(
                &pick_response.market_cap_at_call,
                &pick_response.current_market_cap,
            )
            .round_dp(2)
            .to_string()
            .parse::<f32>()
            .unwrap_or_default();

            // Update best pick and total returns
            pick_responses.push(pick_response);
        }

        debug!("Cache miss for key: {}", cache_key);
        self.redis_service
            .set_cached(&cache_key, &pick_responses, 300)
            .await
            .map_err(|e| {
                error!("Failed to cache token picks: {}", e);
                ApiError::InternalServerError("Failed to cache token picks".to_string())
            })?;

        if !picks_to_update.is_empty() {
            info!("Updating {} token picks", picks_to_update.len());
            if let Err(e) = self
                .token_repository
                .update_token_picks(picks_to_update.values().cloned().collect())
                .await
            {
                error!("Failed to update token picks: {}", e);
            }
        }
        Ok(pick_responses)
    }

    pub async fn save_token_pick(&self, pick: TokenPickRequest) -> Result<TokenPick, ApiError> {
        info!(
            "Saving token pick for user {} and token {}",
            pick.telegram_user_id, pick.address
        );

        let user = self
            .user_service
            .find_by_telegram_user_id(pick.telegram_user_id.parse().map_err(|e| {
                error!("Failed to parse telegram user id: {}", e);
                ApiError::InternalServerError("Invalid telegram user id".to_string())
            })?)
            .await?
            .ok_or_else(|| {
                tracing::error!("User {} not found", pick.telegram_user_id);
                ApiError::InternalServerError("User not found".to_string())
            })?;

        let token_info = self
            .rust_monorepo_service
            .get_latest_w_metadata(vec![pick.address.clone()])
            .await?;

        let token_info = token_info.get(&pick.address).ok_or_else(|| {
            tracing::error!("Token info not found for address {}", pick.address);
            ApiError::InternalServerError("Token info not found".to_string())
        })?;

        if let Ok(None) = self.token_repository.get_token(&pick.address, None).await {
            let token: Token = token_info.clone().into();
            tracing::debug!("Saving new token: {:?}", token);
            self.token_repository.save_token(token).await?;
        }

        let market_cap_at_call = token_info
            .metadata
            .mc
            .unwrap_or_else(|| token_info.metadata.supply.unwrap_or_default() * token_info.price);

        let call_date = pick
            .timestamp
            .map(|timestamp| chrono::DateTime::from_timestamp(timestamp, 0))
            .unwrap_or_else(|| None);

        let token_pick = TokenPick {
            token: Json(token_info.clone().into()),
            call_date: call_date.unwrap_or(chrono::Utc::now()).into(),
            group_id: pick.telegram_chat_id.parse().map_err(|e| {
                error!("Failed to parse telegram chat id: {}", e);
                ApiError::InternalServerError("Invalid telegram chat id".to_string())
            })?,
            user: Json(user.clone()),
            telegram_message_id: Some(pick.telegram_message_id.parse().map_err(|e| {
                error!("Failed to parse telegram message id: {}", e);
                ApiError::InternalServerError("Invalid telegram message id".to_string())
            })?),
            price_at_call: token_info.price,
            highest_market_cap: Some(market_cap_at_call),
            supply_at_call: token_info.metadata.supply,
            market_cap_at_call,
            ..Default::default()
        };

        debug!("Saving token pick: {:?}", token_pick);
        let token_pick = self.token_repository.save_token_pick(token_pick).await?;

        info!("Successfully saved token pick with id {}", token_pick.id);

        let cache_key = format!("user_picks_stats:{}", user.username);
        if let Err(e) = self.redis_service.delete_cached(&cache_key).await {
            error!("Failed to invalidate cache: {}", e);
        }

        Ok(token_pick)
    }
}

fn calculate_return(market_cap_at_call: &Decimal, highest_market_cap: &Decimal) -> Decimal {
    highest_market_cap / market_cap_at_call
}
