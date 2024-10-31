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
use std::sync::Arc;
use utoipa::{IntoParams, ToSchema};

const TAG: &str = "token";

#[derive(Deserialize, ToSchema, Debug, IntoParams)]
pub struct TokenQuery {
    pub username: Option<String>,
    pub by_group: Option<i64>,
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
) -> Result<(StatusCode, Json<Vec<TokenPickResponse>>), ApiError> {
    let picks = app_state.token_service.list_token_picks(query).await?;

    Ok((
        StatusCode::OK,
        Json(picks.into_iter().map(|p| p.into()).collect()),
    ))
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
