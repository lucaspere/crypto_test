use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct UserStats {
    pub total_picks: i32,
    pub hit_rate: f32,
    pub pick_returns: f32,
    pub realized_profit: f32,
    pub total_volume_traded: f64,
    pub hits: i32,
    pub misses: i32,
    pub best_pick: BestPick,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct BestPick {
    pub token_symbol: String,
    pub multiplier: f32,
}
