use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    external_services::rust_monorepo::get_latest_w_metadata::LatestTokenMetadataResponse,
    models::{
        groups::{Group, GroupSettings},
        profiles::ProfileDetailsResponse,
        token_picks::TokenPickResponse,
        user_stats::UserStats,
    },
    utils::time::TimePeriod,
};

#[derive(Serialize, ToSchema)]
pub struct PaginatedTokenPickResponse {
    pub items: Vec<TokenPickResponse>,
    pub total: i64,
    pub page: u32,
    pub limit: u32,
    pub total_pages: u32,
}

#[derive(Serialize, ToSchema)]
pub struct ProfilePicksAndStatsResponse {
    picks: Vec<TokenPickResponse>,
    stats: UserStats,
}

#[derive(serde::Serialize, ToSchema)]
pub struct PaginatedGroupMembersResponse {
    pub items: Vec<ProfileDetailsResponse>,
    pub total: usize,
    pub page: u32,
    pub limit: u32,
    pub total_pages: u32,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GroupMembersResponse {
    pub members: Vec<ProfileDetailsResponse>,
    pub group_name: String,
    pub group_id: i64,
    pub total: i64,
}

#[derive(Serialize, ToSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct GroupResponse {
    id: i64,
    name: String,
    logo_uri: Option<String>,
    created_at: DateTime<Utc>,
    settings: GroupSettings,
    total_token_pick: i64,
    total_users: i64,
    total_pick_returns: f64,
    average_returns: f64,
    hit_rate: f64,
    realized_profit: u64,
}

impl From<Group> for GroupResponse {
    fn from(group: Group) -> Self {
        GroupResponse {
            id: group.id,
            name: group.name,
            logo_uri: group.logo_uri,
            created_at: group.created_at,
            total_token_pick: group.token_pick_count,
            total_users: group.user_count,
            total_pick_returns: (group.total_returns * 100.0).round() / 100.0,
            average_returns: (group.average_returns * 100.0).round() / 100.0,
            hit_rate: (group.hit_rate * 100.0).round() / 100.0,
            realized_profit: 0,
            settings: group.settings,
        }
    }
}

#[derive(Serialize, ToSchema, Deserialize)]
pub struct LeaderboardResponse {
    pub profiles: Vec<ProfileDetailsResponse>,
}

#[derive(Serialize, ToSchema)]
pub struct LeaderboardGroupResponse(pub Vec<GroupMembersResponse>);

#[derive(Serialize, ToSchema)]
pub struct GroupUserResponse {
    pub group_id: i64,
    pub user_id: Uuid,
    pub joined_at: DateTime<Utc>,
}

#[derive(Serialize, ToSchema)]
pub struct SavedTokenPickResponse {
    pub pick: TokenPickResponse,
    pub has_update: bool,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TokenPickDiff {
    pub market_cap_diff: f32,
    pub price_diff: f32,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TokenPickWithDiffResponse {
    pub pick: TokenPickResponse,
    pub pick_diff: Option<TokenPickDiff>,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum TokenPickResponseType {
    Saved(TokenPickResponse),
    AlreadyCalled(TokenPickWithDiffResponse),
}

#[derive(Serialize, ToSchema)]
pub struct TokenPickResponseWithMetadata {
    #[serde(flatten)]
    pub pick: TokenPickResponseType,
    pub token_metadata: LatestTokenMetadataResponse,
}

#[derive(Debug, Serialize, Default)]
pub struct TokenValueDataResponse {
    pub price: Decimal,
    pub volume: Decimal,
    pub liquidity: Decimal,
    pub time_period: TimePeriod,
    pub price_human_time: String,
}
