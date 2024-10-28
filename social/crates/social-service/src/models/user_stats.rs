use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserStats {
    pub total_picks: i32,
    pub hit_rate: Decimal,
    pub pick_returns: Decimal,
    pub average_pick_return: Decimal,
    pub realized_profit: Decimal,
    pub total_volume_traded: Decimal,
    pub hits: i32,
    pub misses: i32,
    pub best_pick: BestPick,
}

#[derive(Debug, Serialize, Deserialize, Default, ToSchema, Clone)]
pub struct BestPick {
    pub token_symbol: String,
    pub token_address: String,
    pub multiplier: Decimal,
}
