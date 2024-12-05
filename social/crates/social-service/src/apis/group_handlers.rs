use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use futures::future::join_all;
use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    apis::api_models::query::{
        GroupLeaderboardQuery, GroupMembersQuery, GroupPicksQuery, ListGroupMembersQuery,
        ListGroupsQuery,
    },
    models::{
        groups::{CreateOrUpdateGroup, GroupUser},
        token_picks::TokenPickResponse,
    },
    utils::errors::{app_error::AppError, error_payload::ErrorPayload},
    AppState,
};

use super::api_models::{
    request::{AddUserRequest, CreateGroupRequest, TokenGroupQuery},
    response::{
        GroupResponse, GroupUserResponse, LeaderboardGroupResponse, PaginatedGroupMembersResponse,
        PaginatedTokenPickResponse,
    },
};

pub const GROUP_TAG: &str = "groups";

/// Create or update a group
#[utoipa::path(
    post,
    tag = GROUP_TAG,
    path = "/",
    operation_id = "createOrUpdateGroup",
    request_body = CreateGroupRequest,
    responses(
        (status = 200, description = "Group created or updated successfully", body = CreateOrUpdateGroup),
        (status = 400, description = "Bad Request", body = ErrorPayload),
        (status = 500, description = "Internal server error", body = ErrorPayload)
    )
)]
pub(super) async fn create_or_update_group(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<CreateGroupRequest>,
) -> Result<(StatusCode, Json<CreateOrUpdateGroup>), AppError> {
    let group = app_state
        .group_service
        .create_or_update_group(payload)
        .await?;

    Ok((StatusCode::OK, group.into()))
}

/// Get a group by ID
#[utoipa::path(
    get,
    tag = GROUP_TAG,
    path = "/{id}",
    operation_id = "getGroup",
    responses(
        (status = 200, description = "Group retrieved successfully", body = GroupResponse),
        (status = 404, description = "Group not found", body = ErrorPayload),
        (status = 500, description = "Internal server error", body = ErrorPayload)
    ),
    params(
        ("id" = i64, Path, description = "Group ID")
    )
)]
pub(super) async fn get_group(
    State(app_state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let group = app_state.group_service.get_group(id).await?;
    Ok((StatusCode::OK, Json(GroupResponse::from(group))))
}

/// List all groups
#[utoipa::path(
    get,
    tag = GROUP_TAG,
    path = "/list",
    operation_id = "listGroups",
    responses(
        (status = 200, description = "Groups listed successfully", body = Vec<GroupResponse>),
        (status = 404, description = "No groups found", body = ErrorPayload),
        (status = 500, description = "Internal server error", body = ErrorPayload)
    ),
    params(ListGroupsQuery)
)]
pub(super) async fn list_groups(
    State(app_state): State<Arc<AppState>>,
    Query(query): Query<ListGroupsQuery>,
) -> Result<(StatusCode, Json<Vec<GroupResponse>>), AppError> {
    let groups = app_state.group_service.list_groups(&query).await?;

    Ok((
        StatusCode::OK,
        Json(groups.into_iter().map(GroupResponse::from).collect()),
    ))
}

/// Get group picks
#[utoipa::path(
    get,
    tag = GROUP_TAG,
    path = "/{id}/picks",
    operation_id = "getGroupPicks",
    responses(
        (status = 200, description = "Success", body = Vec<PaginatedTokenPickResponse>),
        (status = 404, description = "Group not found", body = ErrorPayload),
        (status = 500, description = "Internal server error", body = ErrorPayload)
    ),
    params(GroupPicksQuery)
)]
pub(super) async fn get_group_picks(
    State(app_state): State<Arc<AppState>>,
    Path(group_id): Path<i64>,
    Query(query): Query<GroupPicksQuery>,
) -> Result<(StatusCode, Json<PaginatedTokenPickResponse>), AppError> {
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

/// Get group members
#[utoipa::path(
    get,
    tag = GROUP_TAG,
    path = "/{id}/members",
    operation_id = "getGroupMembers",
    responses(
        (status = 200, description = "Group members retrieved successfully", body = PaginatedGroupMembersResponse),
        (status = 404, description = "Group not found", body = ErrorPayload),
        (status = 500, description = "Internal server error", body = ErrorPayload)
    ),
    params(GroupMembersQuery)
)]
pub(super) async fn get_group_members(
    State(app_state): State<Arc<AppState>>,
    Path(group_id): Path<i64>,
    Query(query): Query<GroupMembersQuery>,
) -> Result<(StatusCode, Json<PaginatedGroupMembersResponse>), AppError> {
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

/// Add a user to a group
#[utoipa::path(
    post,
    tag = GROUP_TAG,
    path = "/{id}/users",
    operation_id = "addUserToGroup",
    request_body = AddUserRequest,
    responses(
        (status = 200, description = "User added to group successfully", body = GroupUserResponse),
        (status = 400, description = "Bad Request", body = ErrorPayload),
        (status = 404, description = "Group not found", body = ErrorPayload),
        (status = 500, description = "Internal server error", body = ErrorPayload)
    )
)]
pub async fn add_user_to_group(
    State(app_state): State<Arc<AppState>>,
    Path(group_id): Path<i64>,
    Json(payload): Json<AddUserRequest>,
) -> Result<(StatusCode, Json<GroupUserResponse>), AppError> {
    let group_user = app_state
        .group_service
        .add_user_to_group(group_id, &payload)
        .await?;

    let res = GroupUserResponse {
        group_id: group_user.group_id,
        user_id: group_user.user_id,
        joined_at: group_user.joined_at,
    };

    Ok((StatusCode::OK, Json(res)))
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RemoveUserRequest {
    user_id: Uuid,
}

/// Remove a user from a group
#[utoipa::path(
    delete,
    tag = GROUP_TAG,
    path = "/{id}/users",
    request_body = RemoveUserRequest,
    operation_id = "removeUserFromGroup",
    responses(
        (status = 200, description = "Success", body = GroupUser),
        (status = 400, description = "Bad Request", body = ErrorPayload),
        (status = 404, description = "Group not found", body = ErrorPayload),
        (status = 500, description = "Internal server error", body = ErrorPayload)
    )
)]
pub async fn remove_user_from_group(
    State(app_state): State<Arc<AppState>>,
    Path(group_id): Path<i64>,
    Json(payload): Json<RemoveUserRequest>,
) -> Result<(StatusCode, Json<GroupUser>), AppError> {
    let group_user = app_state
        .group_service
        .remove_user_from_group(group_id, payload.user_id)
        .await?;

    Ok((StatusCode::OK, group_user.into()))
}

/// Get group members leaderboard
#[utoipa::path(
    get,
    tag = GROUP_TAG,
    path = "/leaderboard",
    operation_id = "getGroupMembersLeaderboard",
    responses(
        (status = 200, description = "Success", body = LeaderboardGroupResponse),
        (status = 404, description = "Group not found", body = ErrorPayload),
        (status = 500, description = "Internal server error", body = ErrorPayload)
    ),
    params(ListGroupMembersQuery)
)]
pub(super) async fn leaderboard(
    State(app_state): State<Arc<AppState>>,
    Query(params): Query<ListGroupMembersQuery>,
) -> Result<(StatusCode, Json<LeaderboardGroupResponse>), AppError> {
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

/// Get group leaderboard
#[utoipa::path(
    get,
    tag = GROUP_TAG,
    path = "/{id}/leaderboard",
    responses(
        (status = 200, description = "Group leaderboard", body = Vec<TokenPickResponse>),
        (status = 404, description = "Group not found", body = ErrorPayload),
        (status = 500, description = "Internal server error", body = ErrorPayload),
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
) -> Result<(StatusCode, Json<Vec<TokenPickResponse>>), AppError> {
    if !app_state.group_service.group_exists(group_id).await? {
        return Err(AppError::NotFound("Group not found".to_string()));
    }

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
