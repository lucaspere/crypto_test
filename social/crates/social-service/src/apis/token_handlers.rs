use crate::{
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

use super::profile_handlers::TimeRange;

pub const TAG: &str = "token";

#[derive(serde::Serialize, ToSchema)]
pub struct PaginatedTokenPickResponse {
    pub items: Vec<TokenPickResponse>,
    pub total: i64,
    pub page: u32,
    pub limit: u32,
    pub total_pages: u32,
}

#[derive(Debug, Deserialize, IntoParams, Default)]
pub struct TokenQuery {
    pub username: Option<String>,
    pub picked_after: Option<TimeRange>,
    #[param(default = 1)]
    pub page: u32,
    #[param(default = 10)]
    pub limit: u32,
    pub order_by: Option<String>,
    pub order_direction: Option<String>,
    #[param(default = false)]
    pub get_all: Option<bool>,
    pub group_ids: Option<Vec<i64>>,
}

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
    let (picks, total) = app_state.token_service.list_token_picks(query).await?;
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
    pub user_id: Option<Uuid>,
    #[param(default = 1)]
    pub page: u32,
    #[param(default = 10)]
    pub limit: u32,
    pub order_by: Option<String>,
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
