use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::{
    models::{
        groups::{CreateOrUpdateGroup, GroupResponse, GroupUser},
        profiles::ProfileDetailsResponse,
        token_picks::TokenPickResponse,
    },
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

#[utoipa::path(
    post,
    tag = GROUP_TAG,
    path = "/",
    description = "Create or update a group",
    request_body = CreateGroupRequest,
    responses(
        (status = 200, description = "Success", body = CreateOrUpdateGroup),
        (status = 400, description = "Bad Request", body = ErrorResponse),
        (status = 500, description = "Internal Server Error", body = ErrorResponse),
    )
)]
pub(super) async fn create_or_update_group(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<CreateGroupRequest>,
) -> Result<(StatusCode, Json<CreateOrUpdateGroup>), ApiError> {
    let group = app_state
        .group_service
        .create_or_update_group(payload.group_id, &payload.name, &payload.logo_uri)
        .await?;

    Ok((StatusCode::OK, group.into()))
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

    Ok((StatusCode::OK, GroupResponse::from(group).into()))
}

#[utoipa::path(
    get,
    tag = GROUP_TAG,
    path = "/list",
    responses(
        (status = 200, description = "Success", body = Vec<GroupResponse>),
        (status = 404, description = "Not Found", body = ErrorResponse),
        (status = 500, description = "Internal Server Error", body = ErrorResponse),
    )
)]
pub(super) async fn list_groups(
    State(app_state): State<Arc<AppState>>,
) -> Result<(StatusCode, Json<Vec<GroupResponse>>), ApiError> {
    let groups = app_state.group_service.list_groups().await?;

    Ok((
        StatusCode::OK,
        Json(groups.into_iter().map(GroupResponse::from).collect()),
    ))
}

#[derive(Debug, Deserialize, IntoParams, Default)]
pub struct GroupPicksQuery {
    pub username: Option<String>,
    #[param(default = 1)]
    pub page: u32,
    #[param(default = 10)]
    pub limit: u32,
    pub order_by: Option<String>,
    pub order_direction: Option<String>,
}

#[utoipa::path(
    get,
    tag = GROUP_TAG,
    path = "/{id}/picks",
    responses(
        (status = 200, description = "Success", body = Vec<TokenPickResponse>),
    ),
    params(GroupPicksQuery)
)]
pub(super) async fn get_group_picks(
    State(app_state): State<Arc<AppState>>,
    Path(group_id): Path<i64>,
    Query(query): Query<GroupPicksQuery>,
) -> Result<(StatusCode, Json<Vec<TokenPickResponse>>), ApiError> {
    todo!()
}

#[derive(Debug, Deserialize, IntoParams, Default)]
pub struct GroupMembersQuery {
    pub username: Option<String>,
    #[param(default = 1)]
    pub page: u32,
    #[param(default = 10)]
    pub limit: u32,
    pub order_by: Option<String>,
    pub order_direction: Option<String>,
}

#[utoipa::path(
    get,
    tag = GROUP_TAG,
    path = "/{id}/members",
    responses(
        (status = 200, description = "Success", body = Vec<ProfileDetailsResponse>)
    ),
    params(GroupMembersQuery)
)]
pub(super) async fn get_group_members(
    State(app_state): State<Arc<AppState>>,
    Path(group_id): Path<i64>,
    Query(query): Query<GroupMembersQuery>,
) -> Result<(StatusCode, Json<Vec<ProfileDetailsResponse>>), ApiError> {
    todo!()
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AddUserRequest {
    pub user_id: Option<Uuid>,
    pub telegram_id: Option<i64>,
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
        .add_user_to_group(group_id, &payload)
        .await?;

    let res = GroupUserResponse {
        group_id: group_user.group_id,
        user_id: group_user.user_id,
        joined_at: group_user.joined_at,
    };

    Ok((StatusCode::OK, res.into()))
}
#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RemoveUserRequest {
    user_id: Uuid,
}

#[utoipa::path(
    delete,
    tag = GROUP_TAG,
    path = "/{id}/users",
    request_body = RemoveUserRequest,
    description = "Remove a user from a group",
    responses(
        (status = 200, description = "Success", body = GroupUser),
    )
)]
pub async fn remove_user_from_group(
    State(app_state): State<Arc<AppState>>,
    Path(group_id): Path<i64>,
    Json(payload): Json<RemoveUserRequest>,
) -> Result<(StatusCode, Json<GroupUser>), ApiError> {
    let group_user = app_state
        .group_service
        .remove_user_from_group(group_id, payload.user_id)
        .await?;

    Ok((StatusCode::OK, group_user.into()))
}
