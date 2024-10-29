use crate::{
    models::{profiles::ProfileDetailsResponse, token_picks::TokenPickResponse},
    utils::{api_errors::ApiError, ErrorResponse},
    AppState,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
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

#[derive(Deserialize, ToSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TokenPickBody {
    telegram_user_id: String,
    telegram_chat_id: String,
    telegram_message_id: String,
    address: String,
    timestamp: i64,
}

#[utoipa::path(
    post,
    tag = TAG,
    path = "/",
    responses(
        (status = 200, description = "Token picks", body = TokenPickResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    request_body(content = TokenPickBody, content_type = "application/json")
)]
pub(super) async fn post_token_pick(
    State(app_state): State<Arc<AppState>>,
    Json(body): Json<TokenPickBody>,
) -> impl IntoResponse {
    StatusCode::OK.into_response()
}
