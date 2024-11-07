use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

use super::profiles::ProfileDetailsResponse;

#[derive(Clone, Debug, PartialEq, FromRow)]
pub struct Group {
    pub id: i64,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub logo_uri: Option<String>,
    pub token_pick_count: i64,
    pub total_returns: f64,
    pub user_count: i64,
    pub hit_rate: f64,
}

#[derive(Serialize, ToSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct GroupResponse {
    id: i64,
    name: String,
    logo_uri: Option<String>,
    created_at: DateTime<Utc>,
    total_token_pick: i64,
    total_users: i64,
    total_pick_returns: f64,
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
            hit_rate: (group.hit_rate * 100.0).round() / 100.0,
            realized_profit: 0,
        }
    }
}

#[derive(Debug, sqlx::FromRow, Serialize, ToSchema, Default, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrUpdateGroup {
    pub id: i64,
    pub name: String,
    pub logo_uri: Option<String>,
}

impl From<Group> for CreateOrUpdateGroup {
    fn from(group: Group) -> Self {
        CreateOrUpdateGroup {
            id: group.id,
            name: group.name,
            logo_uri: group.logo_uri,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct GroupWithUsers {
    pub group_id: i64,
    pub user_id: Uuid,
    pub joined_at: DateTime<Utc>,
    pub username: String,
}

#[derive(Debug, sqlx::FromRow, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GroupUser {
    pub group_id: i64,
    pub user_id: Uuid,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GroupMembersResponse {
    pub members: Vec<ProfileDetailsResponse>,
}
