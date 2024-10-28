use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use uuid::Uuid;

use crate::AppState;

const TAG: &str = "user";

#[utoipa::path(
    post,
    tag = TAG,
    path = "/{id}/follow/{follower_id}",
    responses(
        (status = 200, description = "User followed successfully"),
        (status = 500, description = "Internal server error")
    ),
    params((
        "id" = Uuid,
        Path,
        description = "User ID"
    ), (
        "follower_id" = Uuid,
        Path,
        description = "Follower ID"
    ))
)]
pub async fn follow_user(
    State(app_state): State<Arc<AppState>>,
    Path((user_id, follower_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    match app_state
        .user_service
        .follow_user(user_id, follower_id)
        .await
    {
        Ok(_) => StatusCode::OK.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

#[utoipa::path(
    post,
     tag = TAG,
    path = "/{id}/unfollow/{follower_id}",
    responses(
        (status = 200, description = "User unfollowed successfully"),
        (status = 500, description = "Internal server error")
    ),
    params((
        "id" = Uuid,
        Path,
        description = "User ID"
    )),
    params((
        "follower_id" = Uuid, Path, description = "Follower ID"
    ))
)]
pub async fn unfollow_user(
    State(app_state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
    Path(follower_id): Path<Uuid>,
) -> impl IntoResponse {
    match app_state
        .user_service
        .unfollow_user(user_id, follower_id)
        .await
    {
        Ok(_) => StatusCode::OK.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
