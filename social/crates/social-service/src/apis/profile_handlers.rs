use crate::{
    apis::api_models::query::ProfileLeaderboardQuery,
    models::{
        profiles::ProfileDetailsResponse,
        token_picks::{ProfilePicksAndStatsQuery, TokenPickResponse},
        user_stats::UserStats,
    },
    utils::{
        api_errors::ApiError,
        time::{default_time_period, TimePeriod},
        ErrorResponse,
    },
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

use super::api_models::response::LeaderboardResponse;

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
    let profile = app_state.profile_service.get_profile(query, None).await?;
    Ok((StatusCode::OK, profile.into()))
}

#[derive(Deserialize, ToSchema, Debug, Clone)]
pub struct ProfileQuery {
    pub username: String,
    #[serde(default = "default_time_period")]
    pub picked_after: TimePeriod,
    pub group_id: Option<i64>,
}

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

#[utoipa::path(
    get,
    tag = TAG,
    path = "/leaderboard",
    responses(
        (status = 200, description = "Leaderboard", body = LeaderboardResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(ProfileLeaderboardQuery)
)]
pub(super) async fn leaderboard(
    State(app_state): State<Arc<AppState>>,
    Query(params): Query<ProfileLeaderboardQuery>,
) -> Result<(StatusCode, Json<LeaderboardResponse>), ApiError> {
    if params.username.is_none() && params.following {
        return Err(ApiError::BadRequest(
            "Cannot use following without username".to_string(),
        ));
    }

    let leaderboard = app_state.profile_service.list_profiles(&params).await?;
    Ok((StatusCode::OK, leaderboard.into()))
}
