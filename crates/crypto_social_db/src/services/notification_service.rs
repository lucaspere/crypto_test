use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct NotificationPreferences {
    pub muted: bool,
    pub notify_follower_calls: bool,
    pub notify_new_points: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Notification {
    pub user_id: Uuid,
    pub message: String,
    pub notification_type: String,
}

pub struct NotificationService {
    redis: Arc<redis::Client>,
}

impl NotificationService {
    pub fn new(redis_url: &str) -> Result<Self, redis::RedisError> {
        let client = redis::Client::open(redis_url)?;
        Ok(Self {
            redis: Arc::new(client),
        })
    }

    pub async fn set_notification_preferences(
        &self,
        user_id: Uuid,
        preferences: &NotificationPreferences,
    ) -> Result<(), redis::RedisError> {
        let mut conn = self.redis.get_multiplexed_async_connection().await?;
        let prefs_json = serde_json::to_string(preferences).unwrap();
        conn.set(format!("user:{}:notification_prefs", user_id), prefs_json)
            .await?;
        Ok(())
    }

    pub async fn get_notification_preferences(
        &self,
        user_id: Uuid,
    ) -> Result<NotificationPreferences, redis::RedisError> {
        let mut conn = self.redis.get_multiplexed_async_connection().await?;
        let prefs_json: String = conn
            .get(format!("user:{}:notification_prefs", user_id))
            .await?;
        Ok(serde_json::from_str(&prefs_json).unwrap())
    }

    pub async fn add_notification(
        &self,
        notification: &Notification,
    ) -> Result<(), redis::RedisError> {
        let mut conn = self.redis.get_multiplexed_async_connection().await?;
        let notification_json = serde_json::to_string(notification).unwrap();
        conn.lpush("notifications", notification_json).await?;
        Ok(())
    }

    pub async fn get_notifications(&self) -> Result<Vec<Notification>, redis::RedisError> {
        let mut conn = self.redis.get_multiplexed_async_connection().await?;
        let notifications: Vec<String> = conn.lrange("notifications", 0, -1).await?;
        Ok(notifications
            .into_iter()
            .map(|n| serde_json::from_str(&n).unwrap())
            .collect())
    }
}
