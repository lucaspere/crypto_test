use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

use crate::services::{notification_service::NotificationPreferences, user_service::UserService};

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
    State(user_service): State<UserService>,
    Path((user_id, follower_id)): Path<(Uuid, Uuid)>,
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

#[utoipa::path(
    get,
    tag = TAG,
    path = "/{id}/notification_preferences",
    responses(
        (status = 200, description = "Notification preferences retrieved successfully", body = NotificationPreferences),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("id" = Uuid, Path, description = "User ID")
    )
)]
pub async fn get_notification_preferences(
    State(user_service): State<UserService>,
    Path(user_id): Path<Uuid>,
) -> impl IntoResponse {
    match user_service.get_notification_preferences(user_id).await {
        Ok(preferences) => (StatusCode::OK, Json(preferences)).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

#[utoipa::path(
    post,
    tag = TAG,
    path = "/{id}/notification_preferences",
    request_body = NotificationPreferences,
    responses(
        (status = 200, description = "Notification preferences updated successfully"),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("id" = Uuid, Path, description = "User ID")
    )
)]
pub async fn set_notification_preferences(
    State(user_service): State<UserService>,
    Path(user_id): Path<Uuid>,
    Json(preferences): Json<NotificationPreferences>,
) -> impl IntoResponse {
    match user_service
        .set_notification_preferences(user_id, &preferences)
        .await
    {
        Ok(_) => StatusCode::OK.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
