use crate::{
    apis::api_models::query::ProfileLeaderboardQuery,
    models::{
        profiles::ProfileDetailsResponse,
        token_picks::{ProfilePicksAndStatsQuery, TokenPickResponse},
        user_stats::UserStats,
    },
    utils::{
        errors::{app_error::AppError, error_payload::ErrorPayload},
        time::{default_time_period, TimePeriod},
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

pub const TAG: &str = "profiles";

/// Get profile details by username
#[utoipa::path(
    get,
    tag = TAG,
    path = "/",
    operation_id = "getProfile",
    responses(
        (status = 200, description = "Profile details retrieved successfully", body = ProfileDetailsResponse),
        (status = 404, description = "User not found", body = ErrorPayload),
        (status = 500, description = "Internal server error", body = ErrorPayload)
    ),
    params(
        ("username" = String, Query, description = "Username")
    )
)]
pub(super) async fn get_profile(
    State(app_state): State<Arc<AppState>>,
    Query(query): Query<ProfileQuery>,
) -> Result<(StatusCode, Json<ProfileDetailsResponse>), AppError> {
    let query = ProfileQuery {
        username: query.username.clone(),
        picked_after: TimePeriod::AllTime,
        ..query
    };
    let profile = app_state.profile_service.get_profile(query, None).await?;
    Ok((StatusCode::OK, profile.into()))
}

#[derive(Deserialize, ToSchema, Debug, Clone)]
pub struct ProfileQuery {
    pub username: String,
    #[serde(default = "default_time_period")]
    pub picked_after: TimePeriod,
    pub group_ids: Option<Vec<i64>>,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct ProfilePicksAndStatsResponse {
    picks: Vec<TokenPickResponse>,
    stats: UserStats,
}

/// Get user picks and stats
#[utoipa::path(
    get,
    tag = TAG,
    path = "/user-picks-and-stats",
    operation_id = "getUserPicksAndStats",
    responses(
        (status = 200, description = "User picks and stats retrieved successfully", body = ProfilePicksAndStatsResponse),
        (status = 500, description = "Internal server error", body = ErrorPayload)
    ),
    params(ProfilePicksAndStatsQuery)
)]
pub(super) async fn get_profile_picks_and_stats(
    State(app_state): State<Arc<AppState>>,
    Query(params): Query<ProfilePicksAndStatsQuery>,
) -> Result<(StatusCode, Json<ProfilePicksAndStatsResponse>), AppError> {
    let (picks, stats) = app_state
        .profile_service
        .get_user_picks_and_stats(&params)
        .await?;

    Ok((
        StatusCode::OK,
        Json(ProfilePicksAndStatsResponse { picks, stats }),
    ))
}

/// Get leaderboard
#[utoipa::path(
    get,
    tag = TAG,
    path = "/leaderboard",
    operation_id = "getLeaderboard",
    responses(
        (status = 200, description = "Leaderboard retrieved successfully", body = LeaderboardResponse),
        (status = 400, description = "Invalid request parameters", body = ErrorPayload),
        (status = 500, description = "Internal server error", body = ErrorPayload)
    ),
    params(ProfileLeaderboardQuery)
)]
pub(super) async fn leaderboard(
    State(app_state): State<Arc<AppState>>,
    Query(params): Query<ProfileLeaderboardQuery>,
) -> Result<(StatusCode, Json<LeaderboardResponse>), AppError> {
    if params.username.is_none() && params.following {
        return Err(AppError::BadRequest(
            "Cannot use following without username".to_string(),
        ));
    }
    let mut params = params.clone();
    if params.filter_by_group {
        if let Some(user_id) = params.user_id {
            let user_groups = app_state.group_service.get_user_groups(user_id).await?;

            params.group_ids = Some(user_groups.iter().map(|g| g.id).collect());
        }
    }

    let leaderboard = app_state.profile_service.list_profiles(&params).await?;
    Ok((StatusCode::OK, leaderboard.into()))
}
