use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;

use crate::{
    apis::api_models::{
        query::TokenQuery, request::TokenGroupQuery, response::PaginatedTokenPickResponse,
    },
    models::{token_picks::TokenPickResponse, tokens::TokenPickRequest},
    utils::errors::{app_error::AppError, error_payload::ErrorPayload},
    AppState,
};

use super::api_models::request::{DeleteTokenPickRequest, PaginatedTokenPickGroupResponse};

pub const TAG: &str = "token-picks";

/// List all token picks with pagination
#[utoipa::path(
    get,
    tag = TAG,
    path = "/picks",
    operation_id = "listTokenPicks",
    responses(
        (status = 200, description = "Token picks retrieved successfully", body = PaginatedTokenPickResponse),
        (status = 404, description = "No token picks found", body = ErrorPayload),
        (status = 500, description = "Internal server error", body = ErrorPayload)
    ),
    params(TokenQuery)
)]
pub(super) async fn list_token_picks(
    State(app_state): State<Arc<AppState>>,
    Query(query): Query<TokenQuery>,
) -> Result<(StatusCode, Json<PaginatedTokenPickResponse>), AppError> {
    let limit = query.limit;
    let page = query.page;

    let (picks, total) = app_state
        .token_service
        .list_token_picks(query, Some(false))
        .await?;

    let response = PaginatedTokenPickResponse {
        items: picks.into_iter().map(|p| p.into()).collect(),
        total,
        page,
        limit,
        total_pages: ((total as f64) / (limit as f64)).ceil() as u32,
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Create a new token pick
#[utoipa::path(
    post,
    tag = TAG,
    path = "/picks",
    operation_id = "createTokenPick",
    request_body = TokenPickRequest,
    responses(
        (status = 200, description = "Token pick created successfully", body = TokenPickResponse),
        (status = 400, description = "Invalid request data", body = ErrorPayload),
        (status = 409, description = "User reached maximum number of picks", body = ErrorPayload),
        (status = 500, description = "Internal server error", body = ErrorPayload)
    )
)]
pub(super) async fn create_token_pick(
    State(app_state): State<Arc<AppState>>,
    Json(body): Json<TokenPickRequest>,
) -> Result<(StatusCode, Json<TokenPickResponse>), AppError> {
    let token_pick = app_state.token_service.save_token_pick(body).await?;
    Ok((StatusCode::OK, Json(token_pick)))
}

/// Get token picks grouped by user's groups
#[utoipa::path(
    get,
    tag = TAG,
    path = "/picks/group",
    operation_id = "listGroupTokenPicks",
    responses(
        (status = 200, description = "Token picks by group retrieved successfully", body = PaginatedTokenPickGroupResponse),
        (status = 404, description = "No token picks found", body = ErrorPayload),
        (status = 500, description = "Internal server error", body = ErrorPayload)
    ),
    params(TokenGroupQuery)
)]
pub(super) async fn list_group_token_picks(
    State(app_state): State<Arc<AppState>>,
    Query(query): Query<TokenGroupQuery>,
) -> Result<(StatusCode, Json<PaginatedTokenPickGroupResponse>), AppError> {
    let limit = query.limit;
    let page = query.page;

    let (picks, total) = app_state
        .token_service
        .list_token_picks_group(query)
        .await?;

    let response = PaginatedTokenPickGroupResponse {
        items: picks,
        total,
        page,
        limit,
        total_pages: ((total as f64) / (limit as f64)).ceil() as u32,
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Delete a token pick
#[utoipa::path(
    delete,
    tag = TAG,
    path = "/picks/{id}",
    operation_id = "deleteTokenPick",
    request_body = DeleteTokenPickRequest,
    responses(
        (status = 200, description = "Token pick deleted successfully"),
        (status = 400, description = "Invalid request data", body = ErrorPayload),
        (status = 404, description = "Token pick not found", body = ErrorPayload),
        (status = 409, description = "Cannot delete pick after time limit", body = ErrorPayload),
        (status = 500, description = "Internal server error", body = ErrorPayload)
    )
)]
pub(super) async fn delete_token_pick(
    State(app_state): State<Arc<AppState>>,
    Json(body): Json<DeleteTokenPickRequest>,
) -> Result<StatusCode, AppError> {
    app_state.token_service.delete_token_pick(body).await?;
    Ok(StatusCode::OK)
}
