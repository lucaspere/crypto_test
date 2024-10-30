use crate::{
    models::{
        profiles::ProfileDetailsResponse, token_picks::TokenPickResponse, tokens::TokenPickRequest,
    },
    utils::{api_errors::ApiError, ErrorResponse},
    AppState,
};
use axum::{extract::State, http::StatusCode, Json};
use serde::Deserialize;
use std::sync::Arc;
use utoipa::ToSchema;

const TAG: &str = "token";

#[utoipa::path(
    get,
    tag = TAG,
    path = "/",
    responses(
        (status = 200, description = "Profile details", body = ProfileDetailsResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params((
        "username" = String,
        Query,
        description = "Username"
    ))
)]
pub(super) async fn get_token_picks(
    State(app_state): State<Arc<AppState>>,
) -> Result<(StatusCode, Json<TokenPickResponse>), ApiError> {
    Err(ApiError::UserNotFound)
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
