use crate::models::tiers::TiersType;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use super::user_stats::{BestPick, UserStats};

pub struct Profile;
/// A summary of a user's picks (calls).
#[derive(Deserialize, Serialize, ToSchema, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProfilePickSummary {
    /// The sum total of all of a user's picks expressed as an integer.
    pub total_picks: i32,
    /// Percentage of the user's picks that have achieved a return of 2x or more since they were made.
    pub hit_rate: Decimal,
    /// Total returns of a user's picks expressed as a multiple.
    pub pick_returns: Decimal,
    /// Average return of a user's picks expressed as a multiple.
    pub average_pick_return: Decimal,
    /// Total realized PnL for that user's Bullpen wallet expressed as a dollar amount.
    pub realized_profit: Decimal,
    /// [BestPick] performing pick.
    pub best_pick: BestPick,
}

impl From<UserStats> for ProfilePickSummary {
    fn from(stats: UserStats) -> Self {
        ProfilePickSummary {
            total_picks: stats.total_picks,
            hit_rate: stats.hit_rate,
            pick_returns: stats.pick_returns,
            average_pick_return: stats.average_pick_return,
            realized_profit: stats.realized_profit,
            best_pick: stats.best_pick,
        }
    }
}

/// A summary of a user's tier.
#[derive(Deserialize, Serialize, ToSchema, Default, Debug)]
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
#[derive(Deserialize, Serialize, ToSchema, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProfileDetailsResponse {
    /// User's ID
    pub id: Uuid,
    /// Bullpen username
    pub username: String,
    /// User's name
    pub name: Option<String>,
    /// User's avatar URL
    pub avatar_url: Option<String>,
    /// User's pick summary
    pub pick_summary: ProfilePickSummary,
    /// User's bio
    pub bio: Option<String>,
    /// User's tier
    pub tier: ProfileTier,
    /// Is the user following the authenticated user
    pub is_following: Option<bool>,
}
