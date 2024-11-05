use std::{collections::HashMap, sync::Arc};

use chrono::Utc;
use futures::future::join_all;
use rust_decimal::{prelude::One, Decimal};
use sqlx::types::Json;
use tracing::{debug, error, info, warn};

use crate::{
    apis::token_handlers::{TokenGroupQuery, TokenQuery},
    external_services::{birdeye::BirdeyeService, rust_monorepo::RustMonorepoService},
    models::{
        groups::CreateOrUpdateGroup,
        token_picks::{TokenPick, TokenPickResponse},
        tokens::{Token, TokenPickRequest},
    },
    repositories::token_repository::{ListTokenPicksParams, TokenRepository},
    services::user_service::UserService,
    utils::api_errors::ApiError,
};

use super::{group_service::GroupService, redis_service::RedisService};

pub struct TokenService {
    token_repository: Arc<TokenRepository>,
    rust_monorepo_service: Arc<RustMonorepoService>,
    user_service: Arc<UserService>,
    redis_service: Arc<RedisService>,
    birdeye_service: Arc<BirdeyeService>,
    group_service: Arc<GroupService>,
}

impl TokenService {
    pub fn new(
        token_repository: Arc<TokenRepository>,
        rust_monorepo_service: Arc<RustMonorepoService>,
        user_service: Arc<UserService>,
        redis_service: Arc<RedisService>,
        birdeye_service: Arc<BirdeyeService>,
        group_service: Arc<GroupService>,
    ) -> Self {
        Self {
            token_repository,
            rust_monorepo_service,
            user_service,
            redis_service,
            birdeye_service,
            group_service,
        }
    }

    pub async fn list_token_picks(
        &self,
        query: TokenQuery,
    ) -> Result<(Vec<TokenPickResponse>, i64), ApiError> {
        debug!("Listing token picks with query: {:?}", query);

        let user = if let Some(username) = &query.username {
            debug!("Looking up user by username: {}", username);
            self.user_service.get_by_username(username).await?
        } else {
            None
        };

        let params = ListTokenPicksParams {
            user_id: user.map(|u| u.id),
            page: query.page,
            limit: query.limit,
            order_by: query.order_by,
            order_direction: query.order_direction,
            get_all: query.get_all.unwrap_or(false),
            group_ids: query.group_ids,
        };

        let cache_key = format!(
            "token_picks:user_id={:?}:page={}:limit={}:order_by={:?}:direction={:?}",
            params.user_id, params.page, params.limit, params.order_by, params.order_direction
        );
        debug!("Checking cache for key: {}", cache_key);

        if let Ok(Some(cached)) = self
            .redis_service
            .get_cached::<(Vec<TokenPickResponse>, i64)>(&cache_key)
            .await
        {
            info!("Cache hit for token picks");
            return Ok(cached);
        }

        debug!("Cache miss, fetching token picks from database");

        let (mut picks, total) = if params.group_ids.is_some() {
            info!("Fetching token picks group");
            self.token_repository
                .list_token_picks_group(Some(&params))
                .await
                .map_err(ApiError::from)?
        } else {
            self.token_repository
                .list_token_picks(Some(&params))
                .await
                .map_err(ApiError::from)?
        };

        let token_addresses: Vec<_> = picks.iter().map(|p| p.token.address.clone()).collect();
        debug!(
            "Fetching latest prices for {} tokens",
            token_addresses.len()
        );

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
                        Utc::now().timestamp(),
                        "15m",
                    )
                    .await
                    .map(|result| (pick.token.address.clone(), result))
            }
        });

        debug!("Fetching latest prices and OHLCV data concurrently");
        let (latest_prices, ohlcv_results) =
            tokio::join!(latest_prices_future, join_all(ohlcv_futures));
        let latest_prices = latest_prices?;
        let ohlcv_map: HashMap<_, _> = ohlcv_results.into_iter().filter_map(Result::ok).collect();

        let mut picks_to_update = HashMap::with_capacity(picks.len());
        let mut pick_responses = Vec::with_capacity(picks.len());

        debug!("Processing {} picks", picks.len());
        for pick in &mut picks {
            let latest_price = latest_prices
                .get(&pick.token.address)
                .ok_or_else(|| ApiError::InternalServerError("Price data not found".to_string()))?;

            if !TokenPick::is_qualified(
                latest_price.metadata.mc.unwrap_or_default(),
                latest_price.metadata.liquidity,
                latest_price.metadata.v_24h_usd,
            ) {
                warn!("Token {} is not qualified", pick.token.symbol);
                continue;
            }

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

            let mut pick_response = TokenPickResponse::from(pick.clone());
            pick_response.logo_uri = latest_price.metadata.logo_uri.clone();
            pick_response.current_market_cap =
                latest_price.metadata.mc.unwrap_or_default().round_dp(2);
            pick_response.current_multiplier = 1.2;

            pick_responses.push(pick_response);
        }

        debug!("Caching {} pick responses", pick_responses.len());
        let response = (pick_responses.clone(), total);
        if let Err(e) = self
            .redis_service
            .set_cached(&cache_key, &response, 300)
            .await
        {
            error!("Failed to cache token picks: {}", e);
        }

        if !picks_to_update.is_empty() {
            info!(
                "Updating {} token picks with new market caps",
                picks_to_update.len()
            );
            if let Err(e) = self
                .token_repository
                .update_token_picks(picks_to_update.values().cloned().collect())
                .await
            {
                error!("Failed to update token picks: {}", e);
            }
        }

        debug!("Successfully processed all token picks");
        Ok((pick_responses, total))
    }

    pub async fn save_token_pick(&self, pick: TokenPickRequest) -> Result<TokenPick, ApiError> {
        info!(
            "Saving token pick for user {} and token {}",
            pick.telegram_user_id, pick.address
        );

        let user = self
            .user_service
            .get_by_telegram_user_id(pick.telegram_user_id.parse().map_err(|e| {
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

    pub async fn list_token_picks_group(
        &self,
        query: TokenGroupQuery,
    ) -> Result<(HashMap<String, Vec<TokenPickResponse>>, i64), ApiError> {
        let user = self
            .user_service
            .get_by_id(query.user_id)
            .await?
            .ok_or(ApiError::UserNotFound)?;

        let groups = self.group_service.get_user_groups(user.id).await?;
        if groups.is_empty() {
            return Ok((HashMap::new(), 0));
        }
        info!("Fetching token picks for groups: {:?}", groups);
        let res = self
            .list_token_picks(TokenQuery {
                username: Some(user.username),
                page: query.page,
                limit: query.limit,
                order_by: query.order_by,
                order_direction: query.order_direction,
                get_all: query.get_all,
                group_ids: Some(groups.iter().map(|g| g.id).collect()),
            })
            .await
            .map_err(ApiError::from)?;
        let group_hash: HashMap<i64, &CreateOrUpdateGroup> =
            groups.iter().map(|g| (g.id, g)).collect();
        let map_group_id: HashMap<String, Vec<TokenPickResponse>> =
            res.0.iter().fold(HashMap::new(), |mut acc, pick| {
                if let Some(group) = group_hash.get(&pick.group_id) {
                    acc.entry(group.name.clone())
                        .or_default()
                        .push(pick.clone());
                }
                acc
            });

        Ok((map_group_id, res.1))
    }
}

fn calculate_return(market_cap_at_call: &Decimal, highest_market_cap: &Decimal) -> Decimal {
    if market_cap_at_call.is_zero() || highest_market_cap.is_zero() {
        Decimal::one()
    } else {
        highest_market_cap / market_cap_at_call
    }
}
