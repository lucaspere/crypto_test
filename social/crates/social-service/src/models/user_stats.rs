use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Default, ToSchema)]
pub struct UserStats {
    /// Total number of picks
    pub total_picks: i32,
    /// Percentage of the user's picks that have achieved a return of 2x or more since they were made.
    pub hit_rate: Decimal,
    /// Total return of the user's picks.
    pub pick_returns: Decimal,
    /// Average return of the user's picks.
    pub average_pick_return: Decimal,
    /// Total realized profit of the user's picks.
    pub realized_profit: Decimal,
    /// Total volume traded.
    pub total_volume_traded: Decimal,
    /// Number of hits.
    pub hits: i32,
    /// Number of misses.
    pub misses: i32,
    /// [BestPick] performing pick.
    pub best_pick: BestPick,
}

#[derive(Debug, Serialize, Deserialize, Default, ToSchema, Clone)]
pub struct BestPick {
    /// Token symbol.
    pub token_symbol: String,
    /// Token address.
    pub token_address: String,
    /// The multiplier of the pick.
    pub multiplier: Decimal,
}
