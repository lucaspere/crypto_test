use std::error::Error;

use crate::models::user_stats::{BestPick, UserStats};

#[derive(Clone)]
pub struct ProfileService;

impl ProfileService {
    pub fn new() -> Self {
        ProfileService {}
    }

    pub async fn get_user_stats(&self, username: String) -> Result<UserStats, Box<dyn Error>> {
        let stats = UserStats {
            total_picks: 77,
            hit_rate: 14.0,
            pick_returns: 4.0,
            realized_profit: 4.0,
            total_volume_traded: 24800.0,
            hits: 59,
            misses: 14,
            best_pick: BestPick {
                token_symbol: "$WIF".to_string(),
                multiplier: 2.0,
            },
        };

        Ok(stats)
    }
}
