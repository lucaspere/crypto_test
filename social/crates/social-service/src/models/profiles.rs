use crate::models::tiers::TiersType;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::user_stats::UserStats;

pub struct Profile;
/// A summary of a user's picks (calls).
#[derive(Deserialize, Serialize, ToSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProfilePickSummary {
    /// The sum total of all of a user's picks expressed as an integer.
    total_picks: i32,
    /// Percentage of the user's picks that have achieved a return of 2x or more since they were made.
    hit_hate: Decimal,
    /// Total returns of a user's picks expressed as a multiple.
    pick_returns: Decimal,
    /// Total realized PnL for that user's Bullpen wallet expressed as a dollar amount.
    realized_profit: Decimal,
}

impl From<UserStats> for ProfilePickSummary {
    fn from(stats: UserStats) -> Self {
        ProfilePickSummary {
            total_picks: stats.total_picks,
            hit_hate: stats.hit_rate,
            pick_returns: stats.pick_returns,
            realized_profit: stats.realized_profit,
        }
    }
}

/// A summary of a user's tier.
#[derive(Deserialize, Serialize, ToSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProfileTier {
    /// Total points a user has accumulated.
    total_points: i64,
    /// Points to next tier a user needs to accumulate to reach the next tier.
    points_to_next_tier: i64,
    /// Current tier a user is on.
    current_tier: TiersType,
    /// Next tier a user can reach.
    next_tier: TiersType,
}

/// A user's profile details response.
#[derive(Deserialize, Serialize, ToSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProfileDetailsResponse {
    /// Bullpen username
    pub username: String,
    /// User's name
    pub name: String,
    /// User's avatar URL
    pub avatar_url: String,
    /// User's pick summary
    pub pick_summary: ProfilePickSummary,
    /// User's bio
    pub bio: String,
    /// User's tier
    pub tier: ProfileTier,
}
