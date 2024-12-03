use crate::{
    apis::api_models::query::TokenQuery,
    models::{token_picks::TokenPickResponse, tokens::TokenPickRequest},
    utils::{api_errors::ApiError, ErrorResponse},
    AppState,
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use std::{collections::HashMap, sync::Arc};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use super::api_models::{query::PickLeaderboardSort, response::PaginatedTokenPickResponse};

pub const TAG: &str = "token";

#[utoipa::path(
    get,
    tag = TAG,
    path = "/",
    responses(
        (status = 200, description = "Token picks", body = TokenPickResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(TokenQuery)
)]
pub(super) async fn get_token_picks(
    State(app_state): State<Arc<AppState>>,
    Query(query): Query<TokenQuery>,
) -> Result<(StatusCode, Json<PaginatedTokenPickResponse>), ApiError> {
    let limit = query.limit;
    let page = query.page;

    let (picks, total) = app_state
        .token_service
        .list_token_picks(query, Some(false))
        .await?;
    let total_pages = ((total as f64) / (limit as f64)).ceil() as u32;

    let response = PaginatedTokenPickResponse {
        items: picks.into_iter().map(|p| p.into()).collect(),
        total,
        page,
        limit,
        total_pages,
    };

    Ok((StatusCode::OK, Json(response)))
}

#[derive(Deserialize, ToSchema, Debug)]
pub struct ProfileQuery {
    username: String,
}

#[utoipa::path(
    post,
    tag = TAG,
    path = "/",
    responses(
        (status = 200, description = "Token picks", body = TokenPickResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    request_body(content = TokenPickRequest, content_type = "application/json")
)]
pub(super) async fn post_token_pick(
    State(app_state): State<Arc<AppState>>,
    Json(body): Json<TokenPickRequest>,
) -> Result<(StatusCode, Json<TokenPickResponse>), ApiError> {
    let token_pick = app_state.token_service.save_token_pick(body).await?;

    Ok((StatusCode::OK, Json(token_pick.into())))
}

#[derive(Debug, Deserialize, IntoParams, Default, serde::Serialize, ToSchema)]
pub struct PaginatedTokenPickGroupResponse {
    /// Group name and token picks
    pub items: HashMap<String, Vec<TokenPickResponse>>,
    pub total: i64,
    pub page: u32,
    pub limit: u32,
    pub total_pages: u32,
}

#[derive(Debug, Deserialize, IntoParams, Default)]
pub struct TokenGroupQuery {
    #[serde(deserialize_with = "crate::utils::serde_utils::deserialize_optional_uuid")]
    pub user_id: Option<Uuid>,
    #[param(default = 1)]
    pub page: u32,
    #[param(default = 10)]
    pub limit: u32,
    pub order_by: Option<PickLeaderboardSort>,
    pub order_direction: Option<String>,
    #[param(default = false)]
    pub get_all: Option<bool>,
    pub group_ids: Option<Vec<i64>>,
}

#[utoipa::path(
    get,
    tag = TAG,
    path = "/my-group",
    responses(
        (status = 200, description = "Token picks by group", body = PaginatedTokenPickGroupResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(TokenGroupQuery)
)]
pub async fn get_token_picks_by_group(
    State(app_state): State<Arc<AppState>>,
    Query(query): Query<TokenGroupQuery>,
) -> Result<(StatusCode, Json<PaginatedTokenPickGroupResponse>), ApiError> {
    let limit = query.limit;
    let page = query.page;
    let (picks, total) = app_state
        .token_service
        .list_token_picks_group(query)
        .await?;
    let total_pages = ((total as f64) / (limit as f64)).ceil() as u32;

    let response = PaginatedTokenPickGroupResponse {
        items: picks,
        total,
        page,
        limit,
        total_pages,
    };

    Ok((StatusCode::OK, Json(response)))
}

#[derive(Deserialize, ToSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeleteTokenPickRequest {
    /// Telegram message id
    pub telegram_message_id: i64,
    /// Telegram user id
    pub telegram_user_id: i64,
    /// Telegram group id
    pub telegram_chat_id: i64,
}

#[utoipa::path(
    delete,
    tag = TAG,
    path = "/",
    request_body(content = DeleteTokenPickRequest, content_type = "application/json"),
    responses(
        (status = 200, description = "Token pick deleted"),
        (status = 409, description = "Can only delete picks within 1 minute of creation", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn delete_token_pick(
    State(app_state): State<Arc<AppState>>,
    Json(body): Json<DeleteTokenPickRequest>,
) -> Result<StatusCode, ApiError> {
    app_state.token_service.delete_token_pick(body).await?;
    Ok(StatusCode::OK)
}
