use crate::models::users::UserResponse;
use crate::repositories::user_repository::UserRepository;
use crate::utils::api_errors::ApiError;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct UserService {
    user_repository: Arc<UserRepository>,
}

impl UserService {
    pub fn new(user_repository: Arc<UserRepository>) -> Self {
        Self { user_repository }
    }

    pub async fn find_by_telegram_user_id(
        &self,
        telegram_user_id: i64,
    ) -> Result<Option<UserResponse>, sqlx::Error> {
        let user = self
            .user_repository
            .find_by_telegram_user_id(telegram_user_id)
            .await?;
        Ok(user.map(UserResponse::from))
    }

    pub async fn get_user(&self, id: Uuid) -> Result<Option<UserResponse>, sqlx::Error> {
        let user = self.user_repository.find_by_id(id).await?;
        Ok(user.map(UserResponse::from))
    }

    pub async fn get_user_by_username(
        &self,
        username: &str,
    ) -> Result<Option<UserResponse>, sqlx::Error> {
        let user = self.user_repository.find_by_username(username).await?;
        Ok(user.map(UserResponse::from))
    }

    pub async fn follow_user(&self, follower_id: Uuid, followed_id: Uuid) -> Result<(), ApiError> {
        let follower_user = self.get_followers(follower_id).await?;
        let already_following = follower_user.iter().any(|user| user.id == followed_id);
        if already_following {
            return Err(ApiError::UserAlreadyFollowed);
        }

        self.user_repository
            .follow_user(follower_id, followed_id)
            .await?;

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

    pub async fn get_followers(&self, user_id: Uuid) -> Result<Vec<UserResponse>, sqlx::Error> {
        let followers = self.user_repository.list_followers(user_id).await?;
        Ok(followers.into_iter().map(UserResponse::from).collect())
    }
}
