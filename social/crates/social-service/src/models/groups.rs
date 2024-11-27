use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{types::Json, FromRow};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Clone, Debug, FromRow)]
pub struct Group {
    pub id: i64,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub logo_uri: Option<String>,
    pub is_admin: Option<bool>,
    pub is_active: Option<bool>,
    #[sqlx(json)]
    pub settings: GroupSettings,
    pub token_pick_count: i64,
    pub total_returns: f64,
    pub average_returns: f64,
    pub user_count: i64,
    pub hit_rate: f64,
}

#[derive(Debug, sqlx::FromRow, Serialize, ToSchema, Default, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrUpdateGroup {
    pub id: i64,
    pub name: String,
    pub logo_uri: Option<String>,
    pub is_admin: Option<bool>,
    pub is_active: Option<bool>,
    #[serde(skip_serializing)]
    pub settings: Option<Json<GroupSettings>>,
}

impl From<Group> for CreateOrUpdateGroup {
    fn from(group: Group) -> Self {
        CreateOrUpdateGroup {
            id: group.id,
            name: group.name,
            logo_uri: group.logo_uri,
            is_admin: group.is_admin,
            is_active: group.is_active,
            settings: Some(Json(group.settings)),
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

#[derive(Debug, FromRow, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GroupUser {
    pub group_id: i64,
    pub user_id: Uuid,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, FromRow, Serialize, ToSchema, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct GroupSettings {
    pub privacy: GroupPrivacy,
    pub twitter_metadata: TwitterMetadata,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum GroupPrivacy {
    #[default]
    Public,
    Anonymous,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum TwitterMetadata {
    #[default]
    Enabled,
    Disabled,
}
