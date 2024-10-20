use chrono::{DateTime, FixedOffset};
use rust_decimal::Decimal;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, FromRow)]
pub struct TokenCall {
    pub id: Uuid,
    pub token_address: String,
    pub user_id: Uuid,
    pub group_id: Uuid,
    pub call_type: String,
    pub price_at_call: Decimal,
    pub target_price: Option<Decimal>,
    pub call_date: DateTime<FixedOffset>,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}
