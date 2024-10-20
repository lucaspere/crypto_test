use chrono::{DateTime, FixedOffset};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, FromRow)]
pub struct UserFollow {
    pub follower_id: Uuid,
    pub followed_id: Uuid,
    pub created_at: DateTime<FixedOffset>,
}
