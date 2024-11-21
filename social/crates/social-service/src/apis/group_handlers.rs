use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::{
    apis::api_models::query::{default_limit, GroupMembersQuery, ListGroupMembersQuery},
    models::{
        groups::{CreateOrUpdateGroup, GroupUser},
        token_picks::TokenPickResponse,
    },
    utils::{api_errors::ApiError, ErrorResponse},
    AppState,
};

use super::{
    api_models::{
        query::PickLeaderboardSort,
        request::CreateGroupRequest,
        response::{
            GroupResponse, LeaderboardGroupResponse, PaginatedGroupMembersResponse,
            PaginatedTokenPickResponse,
        },
    },
    token_handlers::TokenGroupQuery,
};

pub const GROUP_TAG: &str = "group";

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
        .create_or_update_group(
            payload.group_id,
            &payload.name,
            &payload.logo_uri,
            &payload.is_admin,
            &payload.is_active,
        )
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

#[derive(Debug, Deserialize, IntoParams, Default)]
pub struct ListGroupsQuery {
    #[serde(deserialize_with = "crate::utils::serde_utils::deserialize_optional_uuid")]
    pub user_id: Option<Uuid>,
}

#[utoipa::path(
    get,
    tag = GROUP_TAG,
    path = "/list",
    responses(
        (status = 200, description = "Success", body = Vec<GroupResponse>),
        (status = 404, description = "Not Found", body = ErrorResponse),
        (status = 500, description = "Internal Server Error", body = ErrorResponse),
    ),
    params(ListGroupsQuery)
)]
pub(super) async fn list_groups(
    State(app_state): State<Arc<AppState>>,
    Query(query): Query<ListGroupsQuery>,
) -> Result<(StatusCode, Json<Vec<GroupResponse>>), ApiError> {
    let groups = app_state.group_service.list_groups(&query).await?;

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
    pub order_by: Option<PickLeaderboardSort>,
    pub order_direction: Option<String>,
}

#[utoipa::path(
    get,
    tag = GROUP_TAG,
    path = "/{id}/picks",
    responses(
        (status = 200, description = "Success", body = Vec<PaginatedTokenPickResponse>),
    ),
    params(GroupPicksQuery)
)]
pub(super) async fn get_group_picks(
    State(app_state): State<Arc<AppState>>,
    Path(group_id): Path<i64>,
    Query(query): Query<GroupPicksQuery>,
) -> Result<(StatusCode, Json<PaginatedTokenPickResponse>), ApiError> {
    let limit = query.limit;
    let page = query.page;

    let picks = app_state
        .token_service
        .list_token_picks_group(TokenGroupQuery {
            group_ids: Some(vec![group_id]),
            limit,
            page,
            order_by: query.order_by,
            order_direction: query.order_direction,
            get_all: None,
            user_id: None,
        })
        .await?;
    let total = picks.1;
    let total_pages = ((total as f64) / (limit as f64)).ceil() as u32;
    let picks = picks.0.into_values().next().unwrap_or(vec![]);
    Ok((
        StatusCode::OK,
        Json(PaginatedTokenPickResponse {
            items: picks,
            total,
            page,
            limit,
            total_pages,
        }),
    ))
}

#[utoipa::path(
    get,
    tag = GROUP_TAG,
    path = "/{id}/members",
    responses(
        (status = 200, description = "Success", body = PaginatedGroupMembersResponse)
    ),
    params(GroupMembersQuery)
)]
pub(super) async fn get_group_members(
    State(app_state): State<Arc<AppState>>,
    Path(group_id): Path<i64>,
    Query(query): Query<GroupMembersQuery>,
) -> Result<(StatusCode, Json<PaginatedGroupMembersResponse>), ApiError> {
    let limit = query.limit;
    let page = query.page;

    let res = app_state
        .group_service
        .list_group_members(group_id, limit, page, query.order_by, query.username)
        .await?;
    let total = res.members.len();
    let total_pages = ((res.total as f64) / (limit as f64)).ceil() as u32;
    Ok((
        StatusCode::OK,
        Json(PaginatedGroupMembersResponse {
            items: res.members,
            total,
            page,
            limit,
            total_pages,
        }),
    ))
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AddUserRequest {
    #[serde(deserialize_with = "crate::utils::serde_utils::deserialize_optional_uuid")]
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

#[utoipa::path(
    get,
    tag = GROUP_TAG,
    path = "/leaderboard",
    responses(
        (status = 200, description = "Success", body = LeaderboardGroupResponse),
    ),
    params(ListGroupMembersQuery)
)]
pub(super) async fn leaderboard(
    State(app_state): State<Arc<AppState>>,
    Query(params): Query<ListGroupMembersQuery>,
) -> Result<(StatusCode, Json<LeaderboardGroupResponse>), ApiError> {
    let user_groups = app_state
        .group_service
        .get_user_groups(params.user_id)
        .await?;

    let group_members_responses = user_groups
        .iter()
        .map(|group| {
            app_state.group_service.list_group_members(
                group.id,
                0,
                0,
                params.sort,
                params.username.clone(),
            )
        })
        .collect::<Vec<_>>();

    let groups = join_all(group_members_responses)
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;

    Ok((StatusCode::OK, Json(LeaderboardGroupResponse(groups))))
}

#[derive(Debug, Deserialize, IntoParams, Clone)]
pub struct GroupLeaderboardQuery {
    #[param(default = 10)]
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[param(default = false)]
    #[serde(default)]
    pub force_refresh: bool,
    #[param(default = "24h")]
    pub timeframe: String,
}

/// Get the top token picks for a specific group
#[utoipa::path(
    get,
    tag = GROUP_TAG,
    path = "/{id}/leaderboard",
    responses(
        (status = 200, description = "Group leaderboard", body = Vec<TokenPickResponse>),
        (status = 404, description = "Group not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    params(
        ("id" = i64, Path, description = "Group ID"),
        GroupLeaderboardQuery
    )
)]
pub async fn get_group_leaderboard(
    State(app_state): State<Arc<AppState>>,
    Path(group_id): Path<i64>,
    Query(query): Query<GroupLeaderboardQuery>,
) -> Result<(StatusCode, Json<Vec<TokenPickResponse>>), ApiError> {
    if !app_state.group_service.group_exists(group_id).await? {
        return Err(ApiError::NotFound("Group not found".to_string()));
    }

    // Force refresh if requested
    if query.force_refresh {
        app_state
            .token_service
            .update_group_leaderboard_cache(group_id, &query)
            .await?;
    }

    let picks = app_state
        .token_service
        .get_group_leaderboard(group_id, &query)
        .await?;

    Ok((StatusCode::OK, Json(picks)))
}
