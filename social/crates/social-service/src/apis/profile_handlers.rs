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
use chrono::{DateTime, Duration, FixedOffset};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::{IntoParams, ToSchema};

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
    let profile = app_state.profile_service.get_profile(query).await?;
    Ok((StatusCode::OK, profile.into()))
}

#[derive(Deserialize, ToSchema, Debug, Clone, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TimeRange {
    Day,
    Week,
    Month,
    #[default]
    Year,
}

impl TimeRange {
    pub fn to_date_time(&self, now: DateTime<FixedOffset>) -> DateTime<FixedOffset> {
        match self {
            TimeRange::Day => now - Duration::days(1),
            TimeRange::Week => now - Duration::weeks(1),
            TimeRange::Month => now - Duration::days(30),
            TimeRange::Year => now - Duration::days(365),
        }
    }
}

impl ToString for TimeRange {
    fn to_string(&self) -> String {
        match self {
            TimeRange::Day => "day".to_string(),
            TimeRange::Week => "week".to_string(),
            TimeRange::Month => "month".to_string(),
            TimeRange::Year => "year".to_string(),
        }
    }
}

#[derive(Deserialize, ToSchema, Debug, Clone)]
pub struct ProfileQuery {
    pub username: String,
    #[serde(default = "default_time_range")]
    pub picked_after: TimeRange,
    pub group_id: Option<i64>,
}

fn default_time_range() -> TimeRange {
    TimeRange::Year
}

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
        .get_user_picks_and_stats(&params, None)
        .await?;

    Ok((
        StatusCode::OK,
        Json(ProfilePicksAndStatsResponse { picks, stats }),
    ))
}
#[derive(Serialize, Deserialize, Debug, Clone, Copy, ToSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum LeaderboardSort {
    #[default]
    PickReturns,
    HitRate,
    RealizedProfit,
    TotalPicks,
    MostRecentPick,
}

#[derive(Deserialize, Serialize, ToSchema, IntoParams, Debug, Default)]
pub struct LeaderboardQuery {
    #[serde(default)]
    pub sort: Option<LeaderboardSort>,
    #[serde(default)]
    pub order: Option<String>,
    #[serde(default = "default_time_range")]
    pub picked_after: TimeRange,
    pub group_id: Option<i64>,
}

#[derive(Serialize, ToSchema)]
pub struct LeaderboardResponse {
    pub profiles: Vec<ProfileDetailsResponse>,
}

#[utoipa::path(
    get,
    tag = TAG,
    path = "/leaderboard",
    responses(
        (status = 200, description = "Leaderboard", body = LeaderboardResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(LeaderboardQuery)
)]
pub(super) async fn leaderboard(
    State(app_state): State<Arc<AppState>>,
    Query(params): Query<LeaderboardQuery>,
) -> Result<(StatusCode, Json<LeaderboardResponse>), ApiError> {
    let leaderboard = app_state.profile_service.list_profiles(&params).await?;
    Ok((StatusCode::OK, leaderboard.into()))
}
