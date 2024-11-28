use std::sync::Arc;

use axum::{
    extract::{Multipart, Path, State},
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
        Err(e) => {
            println!("Error following user: {:?}", e);
            e.into_response()
        }
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
    path = "/{username}/followers",
    responses(
        (status = 200, description = "User followers", body = Vec<UserResponse>)
    ),
    params((
        "username" = String,
        Path,
    ))
)]
pub async fn get_user_followers(
    State(app_state): State<Arc<AppState>>,
    Path(username): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let followers = app_state.user_service.get_followers(&username).await?;

    Ok((StatusCode::OK, Json(followers)))
}

#[utoipa::path(
    post,
    tag = TAG,
    path = "/{telegram_id}/avatar",
    request_body(content_type = "multipart/form-data", content = Vec<u8>),
    responses(
        (status = 200, description = "Avatar uploaded successfully", body = UserResponse),
        (status = 400, description = "Invalid file type"),
        (status = 500, description = "Internal server error")
    ),
    params((
        "telegram_id" = i64,
        Path,
    ))
)]
pub async fn upload_avatar(
    State(app_state): State<Arc<AppState>>,
    Path(user_telegram_id): Path<i64>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, ApiError> {
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?
    {
        let content_type = field
            .content_type()
            .ok_or(ApiError::BadRequest("Missing content type".to_string()))?
            .to_string();
        let data = field
            .bytes()
            .await
            .map_err(|e| ApiError::BadRequest(e.to_string()))?;

        let _avatar_url = app_state
            .s3_service
            .upload_profile_image(&user_telegram_id, data, &content_type)
            .await?;

        let user = app_state
            .user_service
            .get_by_telegram_user_id(user_telegram_id)
            .await?
            .ok_or(ApiError::UserNotFound)?;
        return Ok((StatusCode::OK, Json(user)));
    }

    Err(ApiError::BadRequest("No file provided".to_string()))
}
