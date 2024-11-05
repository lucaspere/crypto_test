use std::sync::Arc;

use uuid::Uuid;

use crate::{
    apis::group_handlers::AddUserRequest,
    models::groups::{CreateOrUpdateGroup, Group, GroupUser},
    repositories::group_repository::GroupRepository,
    utils::api_errors::ApiError,
};

use super::user_service::UserService;

pub struct GroupService {
    repository: Arc<GroupRepository>,
    user_service: Arc<UserService>,
}

impl GroupService {
    pub fn new(repository: Arc<GroupRepository>, user_service: Arc<UserService>) -> Self {
        Self {
            repository,
            user_service,
        }
    }

    pub async fn create_or_update_group(
        &self,
        id: i64,
        name: &str,
        logo_uri: &Option<String>,
    ) -> Result<CreateOrUpdateGroup, ApiError> {
        self.repository
            .upsert_group(id, name, logo_uri)
            .await
            .map_err(|e| ApiError::DatabaseError(e))
    }

    pub async fn get_group(&self, id: i64) -> Result<Option<Group>, ApiError> {
        self.repository
            .get_group(id)
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
            .list_groups(user_id)
            .await
            .map_err(|e| ApiError::DatabaseError(e))
    }
}
