use std::sync::Arc;

use uuid::Uuid;

use crate::{
    apis::{
        api_models::{query::ProfileLeaderboardSort, response::GroupMembersResponse},
        group_handlers::{AddUserRequest, ListGroupsQuery},
        profile_handlers::ProfileQuery,
    },
    models::groups::{CreateOrUpdateGroup, Group, GroupUser},
    repositories::group_repository::GroupRepository,
    utils::{api_errors::ApiError, time::TimePeriod},
};

use super::{profile_service::ProfileService, user_service::UserService};
use futures::future::join_all;

pub struct GroupService {
    repository: Arc<GroupRepository>,
    user_service: Arc<UserService>,
    profile_service: Arc<Option<ProfileService>>,
}

impl GroupService {
    pub fn new(
        repository: Arc<GroupRepository>,
        user_service: Arc<UserService>,
        profile_service: Arc<Option<ProfileService>>,
    ) -> Self {
        Self {
            repository,
            user_service,
            profile_service,
        }
    }

    pub async fn create_or_update_group(
        &self,
        id: i64,
        name: &str,
        logo_uri: &Option<String>,
        is_admin: &Option<bool>,
    ) -> Result<CreateOrUpdateGroup, ApiError> {
        self.repository
            .upsert_group(id, name, logo_uri, is_admin)
            .await
            .map_err(|e| ApiError::DatabaseError(e))
    }

    pub async fn get_group(&self, id: i64) -> Result<Option<Group>, ApiError> {
        self.repository
            .get_group(id)
            .await
            .map_err(|e| ApiError::DatabaseError(e))
    }

    pub async fn list_groups(&self, query: &ListGroupsQuery) -> Result<Vec<Group>, ApiError> {
        self.repository
            .list_groups(query)
            .await
            .map_err(|e| ApiError::DatabaseError(e))
    }

    pub async fn add_user_to_group(
        &self,
        group_id: i64,
        payload: &AddUserRequest,
    ) -> Result<GroupUser, ApiError> {
        let user_id = match (payload.user_id, &payload.telegram_id) {
            (Some(id), _) => id,
            (None, Some(telegram_id)) => {
                let user = self
                    .user_service
                    .get_by_telegram_user_id(*telegram_id)
                    .await?;
                user.unwrap().id
            }
            (None, None) => {
                return Err(ApiError::BadRequest(
                    "Either user_id or telegram_id must be provided".to_string(),
                ))
            }
        };

        self.repository
            .add_user_to_group(group_id, user_id)
            .await
            .map_err(|e| ApiError::DatabaseError(e))
    }

    pub async fn remove_user_from_group(
        &self,
        group_id: i64,
        user_id: Uuid,
    ) -> Result<GroupUser, ApiError> {
        let group_user = self.repository.get_group_user(group_id, user_id).await?;

        if let Some(group_user) = group_user {
            self.repository
                .remove_user_from_group(group_id, user_id)
                .await?;
            Ok(group_user)
        } else {
            Err(ApiError::UserNotFound)
        }
    }

    pub async fn get_user_groups(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<CreateOrUpdateGroup>, ApiError> {
        self.repository
            .list_user_groups(user_id)
            .await
            .map_err(|e| ApiError::DatabaseError(e))
    }

    pub async fn list_group_members(
        &self,
        group_id: i64,
        limit: u32,
        page: u32,
        sort: Option<ProfileLeaderboardSort>,
    ) -> Result<GroupMembersResponse, ApiError> {
        let (group_members, group_name, total) = self
            .repository
            .list_group_members(group_id, limit, page, sort.is_some())
            .await?;

        if let Some(profile_service) = &self.profile_service.as_ref() {
            let profiles = join_all(group_members.iter().map(|g| {
                profile_service.get_profile(
                    ProfileQuery {
                        username: g.username.clone(),
                        picked_after: TimePeriod::AllTime,
                        group_id: Some(group_id),
                    },
                    None,
                )
            }))
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;

            let mut group_members_response = GroupMembersResponse {
                members: profiles,
                group_name,
                group_id: group_id,
                total,
            };

            if let Some(sort) = sort {
                group_members_response.members.sort_by(|a, b| match sort {
                    ProfileLeaderboardSort::PickReturns => b
                        .pick_summary
                        .pick_returns
                        .cmp(&a.pick_summary.pick_returns),
                    ProfileLeaderboardSort::HitRate => {
                        b.pick_summary.hit_rate.cmp(&a.pick_summary.hit_rate)
                    }
                    ProfileLeaderboardSort::RealizedProfit => b
                        .pick_summary
                        .realized_profit
                        .cmp(&a.pick_summary.realized_profit),
                    ProfileLeaderboardSort::TotalPicks => {
                        b.pick_summary.total_picks.cmp(&a.pick_summary.total_picks)
                    }
                    _ => a.username.cmp(&b.username),
                });
            }

            Ok(group_members_response)
        } else {
            Err(ApiError::InternalServerError(
                "Profile service is not available".to_string(),
            ))
        }
    }
}
