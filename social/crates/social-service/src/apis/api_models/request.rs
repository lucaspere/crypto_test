use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateGroupRequest {
    pub group_id: i64,
    pub name: String,
    pub is_admin: Option<bool>,
    pub is_active: Option<bool>,
    pub logo_uri: Option<String>,
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AddUserRequest {
    pub user_id: Option<Uuid>,
    pub telegram_id: Option<i64>,
}
