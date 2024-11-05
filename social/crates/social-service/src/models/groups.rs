use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, FromRow)]
pub struct Group {
    pub id: i64,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub logo_uri: Option<String>,
    pub token_pick_count: i64,
    pub user_count: i64,
    pub hit_rate: i64,
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
    total_pick_returns: u64,
    hit_rate: u64,
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
            total_pick_returns: 0,
            hit_rate: 0,
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

#[derive(Debug, sqlx::FromRow)]
pub struct GroupWithUsers {
    pub group: Group,
    pub users: Vec<GroupUser>,
}

#[derive(Debug, sqlx::FromRow, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GroupUser {
    pub group_id: i64,
    pub user_id: Uuid,
    pub joined_at: DateTime<Utc>,
}
