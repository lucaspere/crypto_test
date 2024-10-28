use chrono::{DateTime, FixedOffset};
use sqlx::FromRow;

#[derive(Clone, Debug, PartialEq, FromRow)]
pub struct Group {
    pub id: i64,
    pub name: String,
    pub created_at: DateTime<FixedOffset>,
}
