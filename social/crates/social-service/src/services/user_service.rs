use crate::models::users::{SavedUser, User, UserResponse};
use crate::repositories::user_repository::UserRepository;
use crate::utils::errors::app_error::AppError;
use std::sync::Arc;
use tracing::info;
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

    pub async fn follow_user(&self, user_id: Uuid, followed_id: Uuid) -> Result<(), AppError> {
        let user = self
            .get_by_id(followed_id)
            .await?
            .ok_or(AppError::NotFound(format!(
                "User with id {} not found",
                followed_id
            )))?;
        let follower_user = self.get_followers(&user.username).await?;
        let already_following = follower_user.iter().any(|user| user.id == user_id);
        if already_following {
            return Err(AppError::BusinessLogicError(
                "User already followed".to_string(),
            ));
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

    pub async fn upsert_user(
        &self,
        telegram_user_id: i64,
        _telegram_chat_id: Option<i64>,
    ) -> Result<(Option<SavedUser>, Option<String>), AppError> {
        if let Some((mut saved_user, photo_id)) = self
            .telegram_service
            .get_user_by_telegram_id(telegram_user_id)
            .await
            .ok()
        {
            let image_data = if let Some(photo_id) = photo_id {
                self.telegram_service
                    .get_user_avatar_by_file_id(&photo_id)
                    .await?
            } else {
                Vec::new()
            };

            if !image_data.is_empty() {
                let avatar_url = self
                    .s3_service
                    .upload_profile_image(&telegram_user_id, image_data.into(), "image/jpeg")
                    .await?;
                info!("Uploaded avatar to {}", avatar_url);
                saved_user.image_uri = Some(avatar_url);
            }
            let user = self.user_repository.save_user(saved_user.clone()).await?;
            saved_user.id = user.unwrap_or_default().id;

            Ok((Some(saved_user), None))
        } else {
            Err(AppError::NotFound(format!(
                "User with telegram id {} not found",
                telegram_user_id
            )))
        }
    }
}
