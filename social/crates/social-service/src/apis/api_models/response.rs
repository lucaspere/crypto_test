use serde::Serialize;
use utoipa::ToSchema;

use crate::models::{
    profiles::ProfileDetailsResponse, token_picks::TokenPickResponse, user_stats::UserStats,
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
