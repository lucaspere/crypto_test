use std::sync::Arc;

use crate::{
    models::{
        picks::UserPicksResponse, profiles::ProfileDetailsResponse, token_picks::TokenPick,
        user_stats::UserStats,
    },
    AppState,
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use utoipa::ToSchema;

const TAG: &str = "profile";

#[utoipa::path(
    get,
    tag = TAG,
    path = "/",
    responses(
        (status = 200, description = "Profile details", body = ProfileDetailsResponse),
        (status = 500, description = "Internal server error")
    ),
    params((
        "username" = String,
        Query,
        description = "Username"
    ))
)]
pub(super) async fn get_profile_details(
    State(app_state): State<Arc<AppState>>,
    Query(username): Query<String>,
) -> impl IntoResponse {
    StatusCode::OK.into_response()
}

#[derive(Deserialize, ToSchema)]
pub struct ProfileQuery {
    username: String,
}

#[utoipa::path(
    get,
    tag = TAG,
    path = "/user-stats",
    responses(
        (status = 200, description = "User stats", body = UserStats),
        (status = 500, description = "Internal server error")
    ),
    params((
        "username" = String,
        Query,
        description = "Username"
    ))
)]
pub(super) async fn get_user_stats(
    State(app_state): State<Arc<AppState>>,
    Query(query): Query<ProfileQuery>,
) -> impl IntoResponse {
    StatusCode::OK.into_response()
}

#[utoipa::path(
    get,
    tag = TAG,
    path = "/user-picks",
    responses(
        (status = 200, description = "User picks", body = UserPicksResponse),
        (status = 500, description = "Internal server error")
    ),
    params((
        "username" = String,
        Query,
        description = "Username"
    ))
)]
pub(super) async fn get_user_picks(
    State(app_state): State<Arc<AppState>>,
    Query(username): Query<String>,
) -> impl IntoResponse {
    StatusCode::OK.into_response()
}

#[derive(Deserialize)]
pub struct ProfilePicksAndStatsResponse {
    picks: Vec<TokenPick>,
    stats: UserStats,
}

#[utoipa::path(
    get,
    tag = TAG,
    path = "/user-picks-and-stats",
    responses(
        (status = 200, description = "User picks and stats"),
        (status = 500, description = "Internal server error")
    ),
    params((
        "username" = String,
        Query,
        description = "Username"
    ), (
        "multiplie",
        Query,
        description = "Multiplier"
    ))
)]
pub(super) async fn get_profile_picks_and_stats(
    State(app_state): State<Arc<AppState>>,
    Query((username, multiplier)): Query<(String, Option<u8>)>,
) -> impl IntoResponse {
    StatusCode::OK.into_response()
}
