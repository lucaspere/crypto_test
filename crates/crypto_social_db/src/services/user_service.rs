use crate::models::users::UserResponse;
use crate::repositories::user_repository::UserRepository;
use std::sync::Arc;
use uuid::Uuid;

use super::notification_service::{Notification, NotificationPreferences, NotificationService};

#[derive(Clone)]
pub struct UserService {
    user_repository: Arc<UserRepository>,
}

impl UserService {
    pub fn new(user_repository: Arc<UserRepository>) -> Self {
        UserService { user_repository }
    }

    pub async fn get_user(&self, id: Uuid) -> Result<Option<UserResponse>, sqlx::Error> {
        let user = self.user_repository.find_by_id(id).await?;
        Ok(user.map(UserResponse::from))
    }

    pub async fn follow_user(
        &self,
        follower_id: Uuid,
        followed_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        self.user_repository
            .follow_user(follower_id, followed_id)
            .await?;

        let followed_user = self.get_user(followed_id).await?.unwrap();
        let follower_user = self.get_user(follower_id).await?.unwrap();

        Ok(())
    }

    pub async fn unfollow_user(
        &self,
        follower_id: Uuid,
        followed_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        self.user_repository
            .unfollow_user(follower_id, followed_id)
            .await
    }

    pub async fn set_notification_preferences(
        &self,
        user_id: Uuid,
        preferences: &NotificationPreferences,
    ) -> Result<(), sqlx::Error> {
        // Implement the logic to save notification preferences in the database
        // You may need to add a new table or column to store these preferences
        Ok(())
    }

    pub async fn get_notification_preferences(
        &self,
        user_id: Uuid,
    ) -> Result<NotificationPreferences, sqlx::Error> {
        // Implement the logic to retrieve notification preferences from the database
        Ok(NotificationPreferences {
            muted: false,
            notify_follower_calls: true,
            notify_new_points: true,
        })
    }
}
