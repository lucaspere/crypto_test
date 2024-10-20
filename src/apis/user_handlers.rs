use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use uuid::Uuid;

use crate::{models::users::UserResponse, services::user_service::UserService};

#[utoipa::path(
    get,
    path = "/users/{id}",
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

// Add other API handlers as needed
