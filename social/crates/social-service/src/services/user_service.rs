use crate::models::users::{User, UserResponse};
use crate::repositories::user_repository::UserRepository;
use crate::utils::api_errors::ApiError;
use std::sync::Arc;
use uuid::Uuid;

use super::s3_service::S3Service;
use super::telegram_service::TeloxideTelegramBotApi;

#[derive(Clone)]
pub struct UserService {
    user_repository: Arc<UserRepository>,
    telegram_service: Arc<TeloxideTelegramBotApi>,
    s3_service: Arc<S3Service>,
}

impl UserService {
    pub fn new(
        user_repository: Arc<UserRepository>,
        telegram_service: Arc<TeloxideTelegramBotApi>,
        s3_service: Arc<S3Service>,
    ) -> Self {
        Self {
            user_repository,
            telegram_service,
            s3_service,
        }
    }

    pub async fn get_by_telegram_user_id(
        &self,
        telegram_user_id: i64,
    ) -> Result<Option<User>, sqlx::Error> {
        let user = self
            .user_repository
            .find_by_telegram_user_id(telegram_user_id)
            .await?;

        Ok(user)
    }

    pub async fn get_by_id(&self, id: Uuid) -> Result<Option<UserResponse>, sqlx::Error> {
        let user = self.user_repository.find_by_id(id).await?;
        Ok(user.map(UserResponse::from))
    }

    pub async fn get_by_username(
        &self,
        username: &str,
    ) -> Result<Option<UserResponse>, sqlx::Error> {
        let user = self.user_repository.find_by_username(username).await?;
        Ok(user.map(UserResponse::from))
    }

    pub async fn follow_user(&self, user_id: Uuid, followed_id: Uuid) -> Result<(), ApiError> {
        let user = self
            .get_by_id(followed_id)
            .await?
            .ok_or(ApiError::UserNotFound)?;
        let follower_user = self.get_followers(&user.username).await?;
        let already_following = follower_user.iter().any(|user| user.id == user_id);
        if already_following {
            return Err(ApiError::UserAlreadyFollowed);
        }

        self.user_repository
            .follow_user(user_id, followed_id)
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

    pub async fn get_followers(&self, username: &str) -> Result<Vec<UserResponse>, sqlx::Error> {
        let followers = self.user_repository.list_followers(username).await?;
        Ok(followers.into_iter().map(UserResponse::from).collect())
    }

    pub async fn list_users(&self) -> Result<Vec<UserResponse>, sqlx::Error> {
        let users = self.user_repository.list_users().await?;
        Ok(users.into_iter().map(UserResponse::from).collect())
    }

    pub fn get_user_avatar(&self, telegram_id: i64) -> String {
        self.s3_service.get_profile_image_url(telegram_id)
    }
}
