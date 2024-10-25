use crate::{
    models::{picks::UserPicksResponse, profiles::ProfileDetailsResponse, user_stats::UserStats},
    services::profile_service::ProfileService,
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
pub async fn get_profile_details(
    State(profile_service): State<ProfileService>,
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
pub async fn get_user_stats(
    State(profile_service): State<ProfileService>,
    Query(username): Query<String>,
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
pub async fn get_user_picks(
    State(profile_service): State<ProfileService>,
    Query(username): Query<String>,
) -> impl IntoResponse {
    StatusCode::OK.into_response()
}
