use chrono::{DateTime, FixedOffset};
use sqlx::FromRow;

#[derive(Clone, Debug, PartialEq, FromRow)]
pub struct Token {
    pub address: String,
    pub name: String,
    pub symbol: String,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}
