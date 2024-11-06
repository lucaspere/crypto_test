use std::{collections::HashMap, sync::Arc};

use chrono::Utc;
use futures::future::join_all;
use rust_decimal::{prelude::One, Decimal};
use sqlx::types::Json;
use tracing::{debug, error, info};

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
            picked_after: query
                .picked_after
                .clone()
                .map(|t| t.to_date_time(Utc::now().into())),
        };

        let cache_key = format!(
            "token_picks:user_id={:?}:page={}:limit={}:order_by={:?}:direction={:?}:picked_after={:?}",
            params.user_id, params.page, params.limit, params.order_by, params.order_direction, query.picked_after.map(|t| t.to_string())
        );

        if let Ok(Some(cached)) = self
            .redis_service
            .get_cached::<(Vec<TokenPickResponse>, i64)>(&cache_key)
            .await
        {
            info!("Cache hit for token picks list");
            return Ok(cached);
        }

        debug!("Cache miss, fetching token picks from database");

        let (picks, total) = if params.group_ids.is_some() {
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

        // Process all picks concurrently
        let pick_futures: Vec<_> = picks
            .iter()
            .map(|pick| {
                let pick_cache_key = format!("token_pick:{}", pick.id);
                self.process_pick_with_cache(pick, pick_cache_key)
            })
            .collect();

        let pick_responses = join_all(pick_futures)
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;

        let response = (pick_responses.clone(), total);
        if let Err(e) = self
            .redis_service
            .set_cached(&cache_key, &response, 600)
            .await
        {
            error!("Failed to cache token picks list: {}", e);
        }

        Ok((pick_responses, total))
    }

    // New helper method to handle caching and processing for a single pick
    async fn process_pick_with_cache(
        &self,
        pick: &TokenPick,
        cache_key: String,
    ) -> Result<TokenPickResponse, ApiError> {
        // Try to get from cache first
        if let Ok(Some(cached_pick)) = self
            .redis_service
            .get_cached::<TokenPickResponse>(&cache_key)
            .await
        {
            return Ok(cached_pick);
        }

        // Process the pick if not in cache
        let processed_pick = self.process_single_pick(pick).await?;

        // Cache the processed pick
        if let Err(e) = self
            .redis_service
            .set_cached(&cache_key, &processed_pick, 600)
            .await
        {
            error!("Failed to cache individual token pick {}: {}", pick.id, e);
        }

        Ok(processed_pick)
    }

    async fn process_single_pick(&self, pick: &TokenPick) -> Result<TokenPickResponse, ApiError> {
        let latest_prices = self
            .rust_monorepo_service
            .get_latest_w_metadata(vec![pick.token.address.clone()])
            .await?;

        let latest_price = latest_prices
            .get(&pick.token.address)
            .ok_or_else(|| ApiError::InternalServerError("Price data not found".to_string()))?;

        // if !TokenPick::is_qualified(
        //     latest_price.metadata.mc.unwrap_or_default(),
        //     latest_price.metadata.liquidity,
        //     latest_price.metadata.v_24h_usd,
        // ) {
        //     warn!("Token {} is not qualified", pick.token.symbol);
        // }

        let current_market_cap = latest_price.metadata.mc.unwrap_or_default();

        // Check if we need to update the highest market cap
        if current_market_cap > pick.highest_market_cap.unwrap_or_default() {
            debug!(
                "Updating highest market cap for token pick {}. Old: {}, New: {}",
                pick.id,
                pick.highest_market_cap.unwrap_or_default(),
                current_market_cap
            );

            // Update in database
            // self.token_repository
            //     .update_highest_market_cap(pick.id, current_market_cap)
            //     .await
            //     .map_err(ApiError::from)?;
        }

        // let ohlcv = self
        //     .birdeye_service
        //     .get_ohlcv_request(
        //         &pick.token.chain,
        //         &pick.token.address,
        //         pick.call_date.timestamp(),
        //         Utc::now().timestamp(),
        //         "15m",
        //     )
        //     .await?;

        let mut pick_response = TokenPickResponse::from(pick.clone());
        pick_response.logo_uri = latest_price.metadata.logo_uri.clone();
        pick_response.current_market_cap = current_market_cap.round_dp(2);
        pick_response.highest_mc_post_call =
            Some(current_market_cap.max(pick.highest_market_cap.unwrap_or_default()));
        pick_response.highest_mult_post_call =
            calculate_return(&pick.market_cap_at_call, &current_market_cap)
                .round_dp(2)
                .to_string()
                .parse::<f32>()
                .unwrap_or_default();

        Ok(pick_response)
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
        let groups = if let Some(user_id) = query.user_id {
            let user = self
                .user_service
                .get_by_id(user_id)
                .await?
                .ok_or(ApiError::UserNotFound)?;

            self.group_service.get_user_groups(user.id).await?
        } else if let Some(group_ids) = query.group_ids {
            let groups = self.group_service.list_groups().await?;
            groups
                .into_iter()
                .filter(|g| group_ids.contains(&g.id))
                .map(CreateOrUpdateGroup::from)
                .collect()
        } else {
            return Ok((HashMap::new(), 0));
        };
        if groups.is_empty() {
            return Ok((HashMap::new(), 0));
        }
        info!("Fetching token picks for groups: {:?}", groups);
        let res = self
            .list_token_picks(TokenQuery {
                username: None,
                page: query.page,
                limit: query.limit,
                order_by: query.order_by,
                order_direction: query.order_direction,
                get_all: query.get_all,
                group_ids: Some(groups.iter().map(|g| g.id).collect()),
                picked_after: None,
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
