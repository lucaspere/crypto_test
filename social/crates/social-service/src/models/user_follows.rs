use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, FromRow, Serialize, Deserialize)]
pub struct UserFollow {
    pub follower_id: Uuid,
    pub followed_id: Uuid,
    pub created_at: DateTime<FixedOffset>,
}
