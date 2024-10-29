use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

pub const TIER_IRON: u64 = 0;
pub const TIER_BRONZE: u64 = 500;
pub const TIER_SILVER: u64 = 2_000;
pub const TIER_GOLD: u64 = 5_000;
pub const TIER_PLATINUM: u64 = 15_000;
pub const TIER_EMERALD: u64 = 40_000;
pub const TIER_DIAMOND: u64 = 100_000;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, ToSchema, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TiersType {
    #[default]
    Iron = TIER_IRON as isize,
    Bronze = TIER_BRONZE as isize,
    Silver = TIER_SILVER as isize,
    Gold = TIER_GOLD as isize,
    Platinum = TIER_PLATINUM as isize,
    Emerald = TIER_EMERALD as isize,
    Diamond = TIER_DIAMOND as isize,
}

impl TiersType {
    pub fn get_next_tier(num: isize) -> Self {
        match num {
            n if n < Self::Bronze as isize => Self::Bronze,
            n if n < Self::Silver as isize => Self::Silver,
            n if n < Self::Gold as isize => Self::Gold,
            n if n < Self::Platinum as isize => Self::Platinum,
            n if n < Self::Emerald as isize => Self::Emerald,
            n if n < Self::Diamond as isize => Self::Diamond,
            _ => Self::Diamond, // If already at Diamond or higher, stay at Diamond
        }
    }

    pub fn get_current_tier(num: isize) -> Self {
        match num {
            n if n < Self::Bronze as isize => Self::Iron,
            n if n < Self::Silver as isize => Self::Bronze,
            n if n < Self::Gold as isize => Self::Silver,
            n if n < Self::Platinum as isize => Self::Gold,
            n if n < Self::Emerald as isize => Self::Platinum,
            n if n < Self::Diamond as isize => Self::Emerald,
            _ => Self::Diamond,
        }
    }
}

pub struct UserMedal {
    user_id: Uuid,
    tier: TiersType,
    earned_at: DateTime<Utc>,
}
