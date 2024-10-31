use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{models::users::UserResponse, utils::api_errors::ApiError, AppState};

const TAG: &str = "user";

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FollowUnfollowUserBody {
    pub follower_id: Uuid,
}

#[utoipa::path(
    post,
    tag = TAG,
    path = "/{id}/follow",
    responses(
        (status = 200, description = "User followed successfully"),
        (status = 500, description = "Internal server error")
    ),
    params((
        "id" = Uuid,
        Path,
    ))
)]
pub async fn follow_user(
    State(app_state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
    Json(body): Json<FollowUnfollowUserBody>,
) -> impl IntoResponse {
    match app_state
        .user_service
        .follow_user(user_id, body.follower_id)
        .await
    {
        Ok(_) => StatusCode::OK.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

#[utoipa::path(
    post,
    tag = TAG,
    path = "/{id}/unfollow",
    responses(
        (status = 200, description = "User unfollowed successfully"),
        (status = 500, description = "Internal server error")
    ),
    params((
        "id" = Uuid,
        Path,
    ))
)]
pub async fn unfollow_user(
    State(app_state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
    Json(body): Json<FollowUnfollowUserBody>,
) -> impl IntoResponse {
    match app_state
        .user_service
        .unfollow_user(user_id, body.follower_id)
        .await
    {
        Ok(_) => StatusCode::OK.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

#[utoipa::path(
    get,
    tag = TAG,
    path = "/{id}/followers",
    responses(
        (status = 200, description = "User followers", body = Vec<UserResponse>)
    ),
    params((
        "id" = Uuid,
        Path,
    ))
)]
pub async fn get_user_followers(
    State(app_state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let followers = app_state.user_service.get_followers(user_id).await?;

    Ok((StatusCode::OK, Json(followers)))
}
