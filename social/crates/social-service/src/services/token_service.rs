use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use chrono::Utc;
use rayon::iter::{IntoParallelIterator, IntoParallelRefMutIterator, ParallelIterator};
use rust_decimal::{prelude::One, Decimal};
use sqlx::types::Json;
use tracing::{debug, error, info};
use uuid::Uuid;

use crate::{
    apis::{
        api_models::query::TokenQuery, group_handlers::GroupLeaderboardQuery,
        token_handlers::TokenGroupQuery,
    },
    external_services::{
        birdeye::BirdeyeService,
        rust_monorepo::{get_latest_w_metadata::LatestTokenMetadataResponse, RustMonorepoService},
    },
    models::{
        groups::CreateOrUpdateGroup,
        token_picks::{TokenPick, TokenPickResponse},
        tokens::{Token, TokenPickRequest},
    },
    repositories::token_repository::{ListTokenPicksParams, TokenRepository, UserPickLimitScope},
    services::user_service::UserService,
    utils::{
        api_errors::ApiError, math::calculate_price_multiplier, redis_keys::RedisKeys,
        time::TimePeriod,
    },
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
    const MAX_PICK_LIMIT: i64 = 5;

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

    fn generate_token_picks_cache_key(&self, params: &ListTokenPicksParams) -> String {
        format!(
            "token_picks:{}:{}:{}:{}:{}:{}:{}:{}",
            params.user_id.unwrap_or(Uuid::nil()),
            params.group_ids.as_ref().map_or("none".to_string(), |ids| {
                ids.iter()
                    .map(|id| id.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            }),
            params.page,
            params.limit,
            params.order_by.unwrap_or_default().to_string(),
            params.order_direction.clone().unwrap_or_default(),
            params
                .picked_after
                .map_or("none".to_string(), |t| t.to_rfc3339()),
            params.following,
        )
    }

    pub async fn list_token_picks(
        &self,
        query: TokenQuery,
    ) -> Result<(Vec<TokenPickResponse>, i64), ApiError> {
        debug!("Listing token picks with query: {:?}", query);
        let mut query = query.clone();
        let user = if let Some(username) = &query.username {
            debug!("Looking up user by username: {}", username);
            self.user_service.get_by_username(username).await?
        } else {
            None
        };
        if query.filter_by_group {
            if let Some(user_id) = query.user_id {
                let user_groups = self.group_service.get_user_groups(user_id).await?;

                query.group_ids = Some(user_groups.iter().map(|g| g.id).collect());
            }
        }
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
            following: query.following.unwrap_or(false),
        };

        let cache_key = self.generate_token_picks_cache_key(&params);

        if let Ok(Some(cached)) = self
            .redis_service
            .get_cached::<(Vec<TokenPickResponse>, i64)>(&cache_key)
            .await
        {
            debug!("Cache hit for token picks list: {}", cache_key);
            return Ok(cached);
        }

        debug!("Cache miss for token picks list: {}", cache_key);

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

        let pick_responses: Vec<_> = picks
            .into_par_iter()
            .map(|pick| TokenPickResponse::from(pick))
            .collect();
        // let pick_futures: Vec<_> = picks
        //     .iter_mut()
        //     .map(|pick| self.process_pick_with_cache(pick))
        //     .collect();

        // let pick_responses = join_all(pick_futures)
        //     .await
        //     .into_par_iter()
        //     .collect::<Result<Vec<_>, _>>()?;

        let response = (pick_responses, total);
        if let Err(e) = self
            .redis_service
            .set_cached(&cache_key, &response, 120)
            .await
        {
            error!("Failed to cache token picks list: {}", e);
        }

        Ok(response)
    }

    async fn process_pick_with_cache(
        &self,
        pick: &mut TokenPick,
    ) -> Result<TokenPickResponse, ApiError> {
        let cache_key = format!("token_pick:{}", pick.id);

        if let Ok(Some(cached_pick)) = self
            .redis_service
            .get_cached::<TokenPickResponse>(&cache_key)
            .await
        {
            return Ok(cached_pick);
        }

        let processed_pick = TokenPickResponse::from(pick.clone());

        if let Err(e) = self
            .redis_service
            .set_cached(&cache_key, &processed_pick, 300)
            .await
        {
            error!("Failed to cache individual token pick {}: {}", pick.id, e);
        }

        Ok(processed_pick)
    }

    pub async fn process_single_pick(
        &self,
        pick: &mut TokenPick,
    ) -> Result<TokenPickResponse, ApiError> {
        let latest_prices = self
            .rust_monorepo_service
            .get_latest_w_metadata(&[pick.token.address.clone()])
            .await
            .map_err(|e| {
                error!(
                    "Failed to fetch latest metadata for token pick {}: {}",
                    pick.id, e
                );
                ApiError::InternalServerError("Failed to fetch latest metadata".to_string())
            })?;

        let latest_price = latest_prices
            .get(&pick.token.address)
            .ok_or_else(|| ApiError::InternalServerError("Price data not found".to_string()))?;

        let current_market_cap = latest_price.price * latest_price.token_info.supply;
        let mut has_update = false;
        if pick.highest_market_cap.unwrap_or_default() == Decimal::ZERO {
            let ohlcv = self
                .birdeye_service
                .get_ohlcv_request(
                    "solana",
                    &pick.token.address,
                    pick.call_date.timestamp(),
                    Utc::now().timestamp(),
                    "1H",
                )
                .await
                .map_err(|e| {
                    error!("Failed to fetch OHLCV for token pick {}: {}", pick.id, e);
                    ApiError::InternalServerError("Failed to fetch OHLCV".to_string())
                })?;

            let supply = pick
                .supply_at_call
                .unwrap_or_else(|| latest_price.token_info.supply);
            let highest_market_cap = ohlcv.high * supply;
            pick.highest_market_cap = if highest_market_cap < Decimal::one() {
                Some(pick.market_cap_at_call)
            } else {
                Some(highest_market_cap)
            };
            let hit_2x = calculate_price_multiplier(&pick.market_cap_at_call, &ohlcv.high)
                >= Decimal::from(2);
            if hit_2x {
                pick.hit_date = Some(Utc::now().into());
            }
            has_update = true;
        }

        // Check if we need to update the highest market cap
        let diff = calculate_price_multiplier(
            &pick.highest_market_cap.unwrap_or_default(),
            &current_market_cap,
        );
        if current_market_cap.round_dp(2) > pick.highest_market_cap.unwrap_or_default().round_dp(2)
            && diff < Decimal::from(20)
        {
            debug!(
                "Updating highest market cap for token pick {}. Old: {}, New: {}. With price {} and supply {}",
                pick.id,
                pick.highest_market_cap.unwrap_or_default(),
                current_market_cap,
                latest_price.price,
                latest_price.token_info.supply
            );

            pick.highest_market_cap = Some(current_market_cap);

            let hit_2x = calculate_price_multiplier(&pick.market_cap_at_call, &current_market_cap)
                >= Decimal::from(2);
            if hit_2x {
                pick.hit_date = Some(Utc::now().into());
            }
            has_update = true;
        }
        pick.highest_multiplier = Some(
            calculate_price_multiplier(
                &pick.market_cap_at_call,
                &pick.highest_market_cap.unwrap_or_default(),
            )
            .round_dp(2),
        );
        let mut pick_response = TokenPickResponse::from(pick.clone());
        pick_response.current_market_cap = current_market_cap.round_dp(2);
        pick_response.current_multiplier =
            calculate_price_multiplier(&pick.market_cap_at_call, &current_market_cap)
                .round_dp(2)
                .to_string()
                .parse::<f32>()
                .unwrap_or_default();

        if has_update || pick_response.highest_mult_post_call > 2.0 {
            info!("Updating highest market cap for token pick {}", pick.id);
            let hit_date = pick.hit_date.take().map(|d| d);
            if let Err(e) = self
                .token_repository
                .update_highest_market_cap(pick.id, pick.highest_market_cap.unwrap(), hit_date)
                .await
            {
                error!(
                    "Failed to update highest market cap for token pick {}: {}",
                    pick.id, e
                );
            }
        }
        Ok(pick_response)
    }

    pub async fn process_single_pick_with_metadata(
        &self,
        pick_row: &mut TokenPick,
        metadata: &LatestTokenMetadataResponse,
    ) -> Result<(TokenPickResponse, bool), ApiError> {
        info!(
            "Processing single pick with metadata for token pick {}",
            pick_row.id
        );
        if !TokenPick::is_qualified(
            metadata.market_cap,
            metadata.metadata.liquidity,
            metadata.metadata.v_24h_usd,
        ) {
            return Ok((TokenPickResponse::from(pick_row.clone()), false));
        }

        let current_market_cap = metadata.price * metadata.token_info.supply;
        let mut has_update = false;

        if pick_row.highest_market_cap.unwrap_or_default() == Decimal::ZERO {
            has_update = self
                .initialize_highest_market_cap(pick_row, metadata)
                .await?;
        }

        has_update |= self.update_highest_market_cap(pick_row, current_market_cap);

        pick_row.highest_multiplier = Some(
            calculate_price_multiplier(
                &pick_row.market_cap_at_call,
                &pick_row.highest_market_cap.unwrap_or_default(),
            )
            .round_dp(2),
        );

        if pick_row.highest_multiplier.unwrap_or_default() > Decimal::from(2) {
            pick_row.hit_date = Some(Utc::now().into());
            has_update = true;
        }

        let mut pick_response = TokenPickResponse::from(pick_row.clone());
        pick_response.current_market_cap = current_market_cap.round_dp(2);
        pick_response.current_multiplier =
            calculate_price_multiplier(&pick_row.market_cap_at_call, &current_market_cap)
                .round_dp(2)
                .to_string()
                .parse::<f32>()
                .unwrap_or_default();

        Ok((pick_response, has_update))
    }

    pub async fn save_token_pick(
        &self,
        pick: TokenPickRequest,
    ) -> Result<TokenPickResponse, ApiError> {
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

        if self.has_user_reached_action_limit(&user.id).await? {
            return Err(ApiError::InternalServerError(
                "User reached the maximum number of picks".to_string(),
            ));
        }
        let group = match pick.telegram_chat_id.parse() {
            Ok(id) => match self.group_service.get_group(id).await {
                Ok(group) => group.into(),
                Err(_) => CreateOrUpdateGroup {
                    id,
                    ..Default::default()
                },
            },
            Err(_) => CreateOrUpdateGroup::default(),
        };
        let token_info = self
            .rust_monorepo_service
            .get_latest_w_metadata(&[pick.address.clone()])
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

        let market_cap_at_call = token_info.market_cap;

        let call_date = pick
            .timestamp
            .map(|timestamp| chrono::DateTime::from_timestamp(timestamp, 0))
            .unwrap_or_else(|| None);

        let token_pick = TokenPick {
            token: token_info.clone().into(),
            call_date: call_date.unwrap_or(chrono::Utc::now()).into(),
            group,
            user: Some(Json(user.clone())),
            telegram_message_id: Some(pick.telegram_message_id.parse().map_err(|e| {
                error!("Failed to parse telegram message id: {}", e);
                ApiError::InternalServerError("Invalid telegram message id".to_string())
            })?),
            price_at_call: token_info.price,
            highest_market_cap: Some(market_cap_at_call),
            supply_at_call: Some(token_info.token_info.supply),
            market_cap_at_call,
            telegram_id: Some(pick.telegram_user_id.parse().map_err(|e| {
                error!("Failed to parse telegram user id: {}", e);
                ApiError::InternalServerError("Invalid telegram user id".to_string())
            })?),
            ..Default::default()
        };

        debug!("Saving token pick: {:?}", token_pick);
        let token_pick = self.token_repository.save_token_pick(token_pick).await?;

        info!("Successfully saved token pick with id {}", token_pick.id);

        let user_cache_key = format!("user_picks_stats:{}", user.username);
        let list_cache_pattern = format!("token_picks:{}:*", user.id);

        self.redis_service.delete_cached(&user_cache_key).await?;
        self.redis_service
            .delete_pattern(&list_cache_pattern)
            .await?;

        let pick_response: TokenPickResponse = token_pick.into();

        Ok(pick_response)
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
            let groups = self.group_service.list_groups(&Default::default()).await?;
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
        let (picks, total) = self
            .list_token_picks(TokenQuery {
                username: None,
                page: query.page,
                limit: query.limit,
                order_by: query.order_by,
                order_direction: query.order_direction,
                get_all: query.get_all,
                group_ids: Some(groups.iter().map(|g| g.id).collect()),
                picked_after: None,
                following: None,
                filter_by_group: false,
                user_id: None,
            })
            .await
            .map_err(ApiError::from)?;

        let group_hash: HashMap<i64, &CreateOrUpdateGroup> =
            groups.iter().map(|g| (g.id, g)).collect();
        let map_group_id: HashMap<String, Vec<TokenPickResponse>> =
            picks.iter().fold(HashMap::new(), |mut acc, pick| {
                if let Some(group) = group_hash.get(&pick.group.id) {
                    acc.entry(group.name.clone())
                        .or_default()
                        .push(pick.clone());
                }
                acc
            });
        Ok((map_group_id, total))
    }

    pub async fn has_user_reached_action_limit(&self, user_id: &Uuid) -> Result<bool, ApiError> {
        let max_limit = UserPickLimitScope::User(*user_id, TimePeriod::Day);

        let count = self
            .token_repository
            .count_user_picks_in_period(max_limit)
            .await?;

        Ok(count >= Self::MAX_PICK_LIMIT)
    }

    pub async fn get_all_tokens(&self) -> Result<HashMap<String, Vec<TokenPick>>, ApiError> {
        info!("Getting all tokens with picks");
        self.token_repository
            .get_all_tokens_with_picks_group_by_group_id()
            .await
            .map_err(ApiError::from)
    }

    pub async fn bulk_update_token_picks(
        &self,
        picks: &[TokenPickResponse],
    ) -> Result<(), ApiError> {
        self.token_repository.bulk_update_token_picks(picks).await?;
        Ok(())
    }

    pub async fn save_many_tokens(&self, tokens: Vec<Token>) -> Result<(), ApiError> {
        self.token_repository.save_many_tokens(tokens).await?;
        Ok(())
    }

    async fn initialize_highest_market_cap(
        &self,
        pick_row: &mut TokenPick,
        metadata: &LatestTokenMetadataResponse,
    ) -> Result<bool, ApiError> {
        let ohlcv = self
            .birdeye_service
            .get_ohlcv_request(
                "solana",
                &metadata.address,
                pick_row.call_date.timestamp(),
                Utc::now().timestamp(),
                "1H",
            )
            .await
            .map_err(|e| {
                error!(
                    "Failed to fetch OHLCV for token pick {}: {}",
                    pick_row.id, e
                );
                ApiError::InternalServerError("Failed to fetch OHLCV".to_string())
            })?;

        let supply = pick_row
            .supply_at_call
            .unwrap_or_else(|| metadata.token_info.supply);
        let highest_market_cap = ohlcv.high * supply;
        pick_row.highest_market_cap = if highest_market_cap < Decimal::one() {
            Some(pick_row.market_cap_at_call)
        } else {
            Some(highest_market_cap)
        };
        Ok(true)
    }

    fn update_highest_market_cap(
        &self,
        pick_row: &mut TokenPick,
        current_market_cap: Decimal,
    ) -> bool {
        let diff = calculate_price_multiplier(
            &pick_row.highest_market_cap.unwrap_or_default(),
            &current_market_cap,
        );

        if current_market_cap.round_dp(2)
            > pick_row.highest_market_cap.unwrap_or_default().round_dp(2)
            && diff < Decimal::from(20)
        {
            info!(
                "Updating highest market cap for token pick {}. Old: {}, New: {}",
                pick_row.id,
                pick_row.highest_market_cap.unwrap_or_default(),
                current_market_cap,
            );
            pick_row.highest_market_cap = Some(current_market_cap);
            return true;
        }
        false
    }

    pub async fn get_group_leaderboard(
        &self,
        group_id: i64,
        query: &GroupLeaderboardQuery,
    ) -> Result<Vec<TokenPickResponse>, ApiError> {
        let zset_key = RedisKeys::get_group_leaderboard_key(group_id, &query.timeframe.to_string());
        let hash_key =
            RedisKeys::get_group_leaderboard_data_key(group_id, &query.timeframe.to_string());

        if let Ok(pick_ids) = self
            .redis_service
            .zrange_by_score(&zset_key, 0, query.limit as isize)
            .await
        {
            if !pick_ids.is_empty() {
                if let Ok(Some(cached_data)) = self
                    .redis_service
                    .hget_multiple::<TokenPickResponse>(&hash_key, &pick_ids)
                    .await
                {
                    return Ok(cached_data);
                }
            }
        }

        let mut picks = self
            .token_repository
            .get_group_leaderboard(group_id, &query.timeframe, query.limit)
            .await?;
        info!("Fetched {} picks", picks.len());
        if picks.is_empty() {
            return Ok(Vec::new());
        }

        info!("Fetching metadata for {} picks", picks.len());
        let addresses: Vec<String> = picks
            .iter()
            .map(|pick| pick.token.address.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        let metadata = self
            .rust_monorepo_service
            .get_latest_w_metadata(&addresses)
            .await?;

        info!(
            "Processing {:?} picks",
            picks.iter().map(|p| p.id).collect::<Vec<_>>()
        );
        let responses: Vec<TokenPickResponse> = picks
            .par_iter_mut()
            .filter_map(|pick| {
                metadata
                    .get(&pick.token.address)
                    .and_then(|token_metadata| {
                        futures::executor::block_on(
                            self.process_single_pick_with_metadata(pick, token_metadata),
                        )
                        .ok()
                        .map(|(response, _)| response)
                    })
            })
            .collect();

        if !responses.is_empty() {
            let query_clone = query.clone();
            let responses_clone = responses.clone();
            let redis_service = self.redis_service.clone();
            tokio::spawn(async move {
                let mut pipe = redis::pipe();
                pipe.atomic();

                for response in &responses_clone {
                    pipe.zadd(
                        &zset_key,
                        response.id.to_string(),
                        response.highest_mult_post_call,
                    );

                    if let Ok(json_data) = serde_json::to_string(&response) {
                        pipe.hset(&hash_key, response.id.to_string(), json_data);
                    }
                }

                let ttl = RedisKeys::get_ttl_for_timeframe(&query_clone.timeframe);
                pipe.expire(&zset_key, ttl);
                pipe.expire(&hash_key, ttl);

                if let Ok(()) = redis_service.execute_pipe(pipe).await {
                    debug!(
                        "Successfully cached group leaderboard for group {} with timeframe {}",
                        group_id, query_clone.timeframe
                    );
                }
            });
        }

        Ok(responses)
    }

    pub async fn update_group_leaderboard_cache(
        &self,
        group_id: i64,
        query: &GroupLeaderboardQuery,
    ) -> Result<(), ApiError> {
        let leaderboard_key =
            RedisKeys::get_group_leaderboard_key(group_id, &query.timeframe.to_string());
        self.redis_service.delete_cached(&leaderboard_key).await?;
        Ok(())
    }
}
