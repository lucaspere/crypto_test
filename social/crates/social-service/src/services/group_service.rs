use std::sync::Arc;

use crate::{
    repositories::group_repository::{Group, GroupRepository, GroupUser},
    utils::api_errors::ApiError,
};
use uuid::Uuid;

pub struct GroupService {
    repository: Arc<GroupRepository>,
}

impl GroupService {
    pub fn new(repository: Arc<GroupRepository>) -> Self {
        Self { repository }
    }

    pub async fn create_or_update_group(
        &self,
        id: i64,
        name: &str,
        logo_uri: &Option<String>,
    ) -> Result<Group, sqlx::Error> {
        self.repository.upsert_group(id, name, logo_uri).await
    }

    pub async fn get_group(&self, id: i64) -> Result<Option<Group>, ApiError> {
        self.repository
            .get_group(id)
            .await
            .map_err(|_| ApiError::InternalServerError(String::from("Failed to get group")))
    }

    pub async fn add_user_to_group(
        &self,
        group_id: i64,
        user_id: Uuid,
    ) -> Result<GroupUser, sqlx::Error> {
        self.repository.add_user_to_group(group_id, user_id).await
    }
}
