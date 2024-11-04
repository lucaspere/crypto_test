use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    utils::{api_errors::ApiError, ErrorResponse},
    AppState,
};

pub const GROUP_TAG: &str = "group";

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateGroupRequest {
    group_id: i64,
    name: String,
    logo_uri: Option<String>,
}

#[derive(Serialize, ToSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct GroupResponse {
    id: i64,
    name: String,
    logo_uri: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[utoipa::path(
    post,
    tag = GROUP_TAG,
    path = "/",
    description = "Create or update a group",
    request_body = CreateGroupRequest,
    responses(
        (status = 200, description = "Success", body = GroupResponse),
        (status = 400, description = "Bad Request", body = ErrorResponse),
        (status = 500, description = "Internal Server Error", body = ErrorResponse),
    )
)]
pub(super) async fn create_or_update_group(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<CreateGroupRequest>,
) -> Result<(StatusCode, Json<GroupResponse>), ApiError> {
    let group = app_state
        .group_service
        .create_or_update_group(payload.group_id, &payload.name, &payload.logo_uri)
        .await?;

    let res = GroupResponse {
        id: group.id,
        name: group.name,
        logo_uri: group.logo_uri,
        created_at: group.created_at,
    };

    Ok((StatusCode::OK, res.into()))
}

#[utoipa::path(
    get,
    tag = GROUP_TAG,
    path = "/{id}",
    responses(
        (status = 200, description = "Success", body = GroupResponse),
        (status = 404, description = "Not Found", body = ErrorResponse),
    ),
    params((
        "id" = i64,
        Path,
        description = "Group ID",
    ))
)]
pub(super) async fn get_group(
    State(app_state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<(StatusCode, Json<GroupResponse>), ApiError> {
    let group = app_state
        .group_service
        .get_group(id)
        .await?
        .ok_or(ApiError::UserNotFound)?;

    let res = GroupResponse {
        id: group.id,
        name: group.name,
        logo_uri: group.logo_uri,
        created_at: group.created_at,
    };

    Ok((StatusCode::OK, res.into()))
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AddUserRequest {
    user_id: Uuid,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GroupUserResponse {
    group_id: i64,
    user_id: Uuid,
    joined_at: chrono::DateTime<chrono::Utc>,
}

#[utoipa::path(
    post,
    tag = GROUP_TAG,
    path = "/{id}/users",
    request_body = AddUserRequest,
    description = "Add a user to a group",
    responses(
        (status = 200, description = "Success", body = GroupUserResponse),
    )
)]
pub async fn add_user_to_group(
    State(app_state): State<Arc<AppState>>,
    Path(group_id): Path<i64>,
    Json(payload): Json<AddUserRequest>,
) -> Result<(StatusCode, Json<GroupUserResponse>), ApiError> {
    let group_user = app_state
        .group_service
        .add_user_to_group(group_id, payload.user_id)
        .await?;

    let res = GroupUserResponse {
        group_id: group_user.group_id,
        user_id: group_user.user_id,
        joined_at: group_user.joined_at,
    };

    Ok((StatusCode::OK, res.into()))
}
