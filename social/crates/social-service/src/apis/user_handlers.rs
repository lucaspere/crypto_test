use std::sync::Arc;

use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use tracing::{error, warn};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    models::users::UserResponse,
    utils::errors::{app_error::AppError, error_payload::ErrorPayload},
    AppState,
};

const TAG: &str = "users";

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FollowUnfollowUserBody {
    pub follower_id: Uuid,
}

/// Follow a user
#[utoipa::path(
    post,
    tag = TAG,
    path = "/{id}/follow",
    operation_id = "followUser",
    responses(
        (status = 200, description = "User followed successfully"),
        (status = 404, description = "User not found", body = ErrorPayload),
        (status = 409, description = "User already followed", body = ErrorPayload),
        (status = 500, description = "Internal server error", body = ErrorPayload)
    ),
    params(
        ("id" = Uuid, Path, description = "User ID to follow")
    ),
    request_body = FollowUnfollowUserBody
)]
pub(super) async fn follow_user(
    State(app_state): State<Arc<AppState>>,
    Path(followed_id): Path<Uuid>,
    Json(body): Json<FollowUnfollowUserBody>,
) -> Result<impl IntoResponse, AppError> {
    app_state
        .user_service
        .follow_user(body.follower_id, followed_id)
        .await?;
    Ok(StatusCode::OK)
}

/// Unfollow a user
#[utoipa::path(
    post,
    tag = TAG,
    path = "/{id}/unfollow",
    operation_id = "unfollowUser",
    responses(
        (status = 200, description = "User unfollowed successfully"),
        (status = 404, description = "User not found", body = ErrorPayload),
        (status = 500, description = "Internal server error", body = ErrorPayload)
    ),
    params(
        ("id" = Uuid, Path, description = "User ID to unfollow")
    ),
    request_body = FollowUnfollowUserBody
)]
pub(super) async fn unfollow_user(
    State(app_state): State<Arc<AppState>>,
    Path(followed_id): Path<Uuid>,
    Json(body): Json<FollowUnfollowUserBody>,
) -> Result<impl IntoResponse, AppError> {
    app_state
        .user_service
        .unfollow_user(body.follower_id, followed_id)
        .await
        .map_err(|e| AppError::DatabaseError(e))?;
    Ok(StatusCode::OK)
}

/// Get followers of a user
#[utoipa::path(
    get,
    tag = TAG,
    path = "/{username}/followers",
    operation_id = "getFollowers",
    responses(
        (status = 200, description = "List of followers", body = Vec<UserResponse>),
        (status = 404, description = "User not found", body = ErrorPayload),
        (status = 500, description = "Internal server error", body = ErrorPayload)
    ),
    params(
        ("username" = String, Path, description = "Username")
    )
)]
pub(super) async fn get_followers(
    State(app_state): State<Arc<AppState>>,
    Path(username): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let followers = app_state
        .user_service
        .get_followers(&username)
        .await
        .map_err(|e| AppError::DatabaseError(e))?;
    Ok((StatusCode::OK, Json(followers)))
}

/// Upload a user's avatar
#[utoipa::path(
    post,
    tag = TAG,
    path = "/avatar/{user_telegram_id}",
    operation_id = "uploadAvatar",
    responses(
        (status = 200, description = "Avatar uploaded successfully", body = UserResponse),
        (status = 400, description = "Invalid file type or missing file", body = ErrorPayload),
        (status = 404, description = "User not found", body = ErrorPayload),
        (status = 500, description = "Internal server error", body = ErrorPayload)
    ),
    params(
        ("user_telegram_id" = i64, Path, description = "User's Telegram ID")
    )
)]
pub(super) async fn upload_avatar(
    State(app_state): State<Arc<AppState>>,
    Path(user_telegram_id): Path<i64>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?
    {
        let content_type = field
            .content_type()
            .ok_or_else(|| AppError::BadRequest("Missing content type".to_string()))?
            .to_string();
        let data = field
            .bytes()
            .await
            .map_err(|e| AppError::BadRequest(e.to_string()))?;

        let _avatar_url = app_state
            .s3_service
            .upload_profile_image(&user_telegram_id, data, &content_type)
            .await?;

        let user = app_state
            .user_service
            .get_by_telegram_user_id(user_telegram_id)
            .await
            .map_err(|e| {
                error!(
                    "Failed to get user by telegram id {}: {}",
                    user_telegram_id, e
                );
                AppError::InternalServerError()
            })?
            .ok_or_else(|| {
                warn!("User with telegram id {} not found", user_telegram_id);
                AppError::NotFound(format!(
                    "User with telegram id {} not found",
                    user_telegram_id
                ))
            })?;

        return Ok((StatusCode::OK, Json(UserResponse::from(user))));
    }

    Err(AppError::BadRequest("No file provided".to_string()))
}
