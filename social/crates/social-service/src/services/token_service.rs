use std::sync::Arc;

use sqlx::types::Json;
use tracing::{debug, error, info};

use crate::{
    apis::token_handlers::TokenQuery,
    external_services::rust_monorepo::RustMonorepoService,
    models::{
        token_picks::TokenPick,
        tokens::{Token, TokenPickRequest},
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
}

impl TokenService {
    pub fn new(
        token_repository: Arc<TokenRepository>,
        rust_monorepo_service: Arc<RustMonorepoService>,
        user_service: Arc<UserService>,
        redis_service: Arc<RedisService>,
    ) -> Self {
        Self {
            token_repository,
            rust_monorepo_service,
            user_service,
            redis_service,
        }
    }

    pub async fn list_token_picks(&self, query: TokenQuery) -> Result<Vec<TokenPick>, ApiError> {
        let user = if let Some(username) = query.username {
            self.user_service.get_user_by_username(&username).await?
        } else {
            None
        };

        let params = ListTokenPicksParams {
            user_id: user.map(|u| u.id),
        };
        debug!("Listing token picks with params: {:?}", params);
        self.token_repository
            .list_token_picks(Some(&params))
            .await
            .map_err(ApiError::from)
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
