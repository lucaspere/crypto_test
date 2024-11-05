use crate::{
    models::{
        profiles::ProfileDetailsResponse,
        token_picks::{ProfilePicksAndStatsQuery, TokenPickResponse},
        user_stats::UserStats,
    },
    utils::{api_errors::ApiError, ErrorResponse},
    AppState,
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

pub const TAG: &str = "profile";

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
pub(super) async fn get_profile(
    State(app_state): State<Arc<AppState>>,
    Query(query): Query<ProfileQuery>,
) -> Result<(StatusCode, Json<ProfileDetailsResponse>), ApiError> {
    let profile = app_state
        .profile_service
        .get_profile(&query.username)
        .await?;

    Ok((StatusCode::OK, profile.into()))
}

#[derive(Deserialize, ToSchema, Debug)]
pub struct ProfileQuery {
    username: String,
}

// #[utoipa::path(
//     get,
//     tag = TAG,
//     path = "/user-stats",
//     responses(
//         (status = 200, description = "User stats", body = UserStats),
//         (status = 500, description = "Internal server error", body = ErrorResponse)
//     ),
//     params((
//         "username" = String,
//         Query,
//         description = "Username"
//     ))
// )]
// pub(super) async fn get_user_stats(
//     State(app_state): State<Arc<AppState>>,
//     Query(query): Query<ProfileQuery>,
// ) -> impl IntoResponse {
//     StatusCode::OK.into_response()
// }

// #[utoipa::path(
//     get,
//     tag = TAG,
//     path = "/user-picks",
//     responses(
//         (status = 200, description = "User picks", body = UserPicksResponse),
//         (status = 500, description = "Internal server error", body = ErrorResponse)
//     ),
//     params((
//         "username" = String,
//         Query,
//         description = "Username"
//     ))
// )]
// pub(super) async fn get_user_picks(
//     State(app_state): State<Arc<AppState>>,
//     Query(username): Query<String>,
// ) -> impl IntoResponse {
//     StatusCode::OK.into_response()
// }
#[derive(Deserialize, Serialize, ToSchema)]
pub struct ProfilePicksAndStatsResponse {
    picks: Vec<TokenPickResponse>,
    stats: UserStats,
}

#[utoipa::path(
    get,
    tag = TAG,
    path = "/user-picks-and-stats",
    responses(
        (status = 200, description = "User picks and stats", body = ProfilePicksAndStatsResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(ProfilePicksAndStatsQuery)
)]
pub(super) async fn get_profile_picks_and_stats(
    State(app_state): State<Arc<AppState>>,
    Query(params): Query<ProfilePicksAndStatsQuery>,
) -> Result<(StatusCode, Json<ProfilePicksAndStatsResponse>), ApiError> {
    let (picks, stats) = app_state
        .profile_service
        .get_user_picks_and_stats(&params)
        .await?;

    Ok((
        StatusCode::OK,
        Json(ProfilePicksAndStatsResponse { picks, stats }),
    ))
}

// #[derive(Deserialize, Serialize, ToSchema)]
// pub struct LeaderboardQuery {
//     page: i64,
//     limit: i64,
// }

// pub(super) async fn leaderboard(
//     State(app_state): State<Arc<AppState>>,
//     Query(params): Query<LeaderboardQuery>,
// ) -> Result<(StatusCode, Json<LeaderboardResponse>), ApiError> {
//     StatusCode::OK.into_response()
// }
