use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::models::groups::GroupSettings;

#[derive(Deserialize, ToSchema, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct CreateGroupRequest {
    pub group_id: i64,
    pub name: String,
    pub is_admin: Option<bool>,
    pub is_active: Option<bool>,
    pub logo_uri: Option<String>,
    pub settings: Option<GroupSettings>,
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AddUserRequest {
    #[serde(deserialize_with = "crate::utils::serde_utils::deserialize_optional_uuid")]
    pub user_id: Option<Uuid>,
    pub telegram_id: Option<i64>,
}
