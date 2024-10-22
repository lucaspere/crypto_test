use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use uuid::Uuid;

use crate::{models::users::UserResponse, services::user_service::UserService};

const TAG: &str = "user";

#[utoipa::path(
    get,
    tag = TAG,
    path = "/{id}",
    responses(
        (status = 200, description = "User found successfully", body = UserResponse),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("id" = Uuid, Path, description = "User ID")
    )
)]
pub async fn get_user(
    State(user_service): State<UserService>,
    Path(user_id): Path<Uuid>,
) -> impl IntoResponse {
    match user_service.get_user(user_id).await {
        Ok(Some(user)) => user.into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

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
    )),
    params((
        "follower_id" = Uuid,
        Path,
        description = "Follower ID"
    ))

)]
pub async fn follow_user(
    State(user_service): State<UserService>,
    Path(user_id): Path<Uuid>,
    Path(follower_id): Path<Uuid>,
) -> impl IntoResponse {
    match user_service.follow_user(user_id, follower_id).await {
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
    State(user_service): State<UserService>,
    Path(user_id): Path<Uuid>,
    Path(follower_id): Path<Uuid>,
) -> impl IntoResponse {
    match user_service.unfollow_user(user_id, follower_id).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

// Add other API handlers as needed
