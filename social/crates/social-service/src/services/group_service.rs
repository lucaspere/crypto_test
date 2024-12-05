use std::sync::Arc;

use bytes::Bytes;
use uuid::Uuid;

use crate::{
    apis::{
        api_models::{
            query::{ListGroupsQuery, ProfileLeaderboardSort},
            request::{AddUserRequest, CreateGroupRequest},
            response::GroupMembersResponse,
        },
        profile_handlers::ProfileQuery,
    },
    models::groups::{CreateOrUpdateGroup, Group, GroupUser},
    repositories::group_repository::GroupRepository,
    utils::{errors::app_error::AppError, time::TimePeriod},
};

use super::{
    profile_service::ProfileService, s3_service::S3Service,
    telegram_service::TeloxideTelegramBotApi, user_service::UserService,
};
use futures::future::join_all;

pub struct GroupService {
    repository: Arc<GroupRepository>,
    user_service: Arc<UserService>,
    profile_service: Arc<Option<ProfileService>>,
    telegram_service: Arc<TeloxideTelegramBotApi>,
    s3_service: Arc<S3Service>,
}

impl GroupService {
    pub fn new(
        repository: Arc<GroupRepository>,
        user_service: Arc<UserService>,
        profile_service: Arc<Option<ProfileService>>,
        telegram_service: Arc<TeloxideTelegramBotApi>,
        s3_service: Arc<S3Service>,
    ) -> Self {
        Self {
            repository,
            user_service,
            profile_service,
            telegram_service,
            s3_service,
        }
    }

    pub async fn create_or_update_group(
        &self,
        payload: CreateGroupRequest,
    ) -> Result<CreateOrUpdateGroup, AppError> {
        let payload = match payload.logo_uri {
            Some(_) => payload,
            None => {
                let mut payload = payload.clone();
                let telegram_user = self
                    .telegram_service
                    .get_username_image_by_telegram_id(payload.group_id)
                    .await
                    .ok();
                if let Some((username, image, _)) = telegram_user {
                    if let Some(image) = image {
                        let logo_uri = self
                            .s3_service
                            .upload_profile_image(
                                &payload.group_id,
                                Bytes::from(image),
                                "image/jpeg",
                            )
                            .await
                            .ok();
                        payload.logo_uri = logo_uri;
                    }
                    payload.name = username;
                    payload
                } else {
                    payload
                }
            }
        };

        self.repository
            .upsert_group(
                payload.group_id,
                &payload.name,
                &payload.logo_uri,
                &payload.is_admin,
                &payload.is_active,
                &payload.settings.unwrap_or_default(),
            )
            .await
            .map_err(AppError::DatabaseError)
    }

    pub async fn get_group(&self, id: i64) -> Result<Group, AppError> {
        self.repository
            .get_group(id)
            .await?
            .ok_or(AppError::NotFound("Group not found".to_string()))
    }

    pub async fn list_groups(&self, query: &ListGroupsQuery) -> Result<Vec<Group>, AppError> {
        self.repository
            .list_groups(query)
            .await
            .map_err(|e| AppError::DatabaseError(e))
    }

    pub async fn add_user_to_group(
        &self,
        group_id: i64,
        payload: &AddUserRequest,
    ) -> Result<GroupUser, AppError> {
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
                return Err(AppError::BadRequest(
                    "Either user_id or telegram_id must be provided".to_string(),
                ))
            }
        };

        self.get_group(group_id).await?;

        self.repository
            .add_user_to_group(group_id, user_id)
            .await
            .map_err(|e| AppError::DatabaseError(e))
    }

    pub async fn remove_user_from_group(
        &self,
        group_id: i64,
        user_id: Uuid,
    ) -> Result<GroupUser, AppError> {
        self.get_group(group_id).await?;

        let group_user = self.repository.get_group_user(group_id, user_id).await?;

        if let Some(group_user) = group_user {
            self.repository
                .remove_user_from_group(group_id, user_id)
                .await?;
            Ok(group_user)
        } else {
            Err(AppError::NotFound(format!(
                "User with id {} not found in group {}",
                user_id, group_id
            )))
        }
    }

    pub async fn get_user_groups(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<CreateOrUpdateGroup>, AppError> {
        self.repository
            .list_user_groups(user_id)
            .await
            .map_err(|e| AppError::DatabaseError(e))
    }

    pub async fn list_group_members(
        &self,
        group_id: i64,
        limit: u32,
        page: u32,
        sort: Option<ProfileLeaderboardSort>,
        username: Option<String>,
    ) -> Result<GroupMembersResponse, AppError> {
        let (group_members, group_name, total) = self
            .repository
            .list_group_members(group_id, limit, page, sort.is_some())
            .await?;

        let user = if let Some(username) = username {
            self.user_service.get_by_username(&username).await?
        } else {
            None
        };
        let user_id = user.map(|u| u.id);
        if let Some(profile_service) = &self.profile_service.as_ref() {
            let profiles = join_all(group_members.iter().map(|g| {
                profile_service.get_profile(
                    ProfileQuery {
                        username: g.username.clone(),
                        picked_after: TimePeriod::AllTime,
                        group_ids: Some(vec![group_id]),
                    },
                    user_id,
                )
            }))
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;

            let mut group_members_response = GroupMembersResponse {
                members: profiles,
                group_name,
                group_id,
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
                    ProfileLeaderboardSort::AverageReturn => b
                        .pick_summary
                        .average_pick_return
                        .cmp(&a.pick_summary.average_pick_return),
                    _ => a.username.cmp(&b.username),
                });
            }

            Ok(group_members_response)
        } else {
            Err(AppError::InternalServerError())
        }
    }

    pub async fn group_exists(&self, group_id: i64) -> Result<bool, AppError> {
        self.repository.group_exists(group_id).await.map_err(|e| {
            if e.to_string().contains("sqlx::error::RowNotFound") {
                AppError::NotFound("Group not found".to_string())
            } else {
                AppError::DatabaseError(e)
            }
        })
    }
}
