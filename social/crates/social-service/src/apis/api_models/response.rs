use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::models::{
    groups::{Group, GroupSettings},
    profiles::ProfileDetailsResponse,
    token_picks::TokenPickResponse,
    user_stats::UserStats,
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
