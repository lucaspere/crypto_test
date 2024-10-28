use crate::models::medals::TiersType;
use serde::Deserialize;
use utoipa::ToSchema;

pub struct Profile;
#[derive(Deserialize, ToSchema)]
pub struct ProfilePickSummary {
    total_picks: i32,
    hit_hate: i16,
    pick_return: i16,
    realized_profit: i16,
}

#[derive(Deserialize, ToSchema)]
pub struct ProfileMedal {
    total_points: i64,
    points_to_next_tier: i64,
    current_tier: TiersType,
    next_tier: TiersType,
}

#[derive(Deserialize, ToSchema)]
pub struct ProfileDetailsResponse {
    username: String,
    avatar_url: String,
    pick_summary: ProfilePickSummary,
    bio: String,
    medal: ProfileMedal,
}
