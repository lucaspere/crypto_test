use std::collections::HashMap;

use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::models::{groups::GroupSettings, token_picks::TokenPickResponse};

use super::query::PickLeaderboardSort;

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

#[derive(Deserialize, ToSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeleteTokenPickRequest {
    /// Telegram message id
    pub telegram_message_id: i64,
    /// Telegram user id
    pub telegram_user_id: i64,
    /// Telegram group id
    pub telegram_chat_id: i64,
}

#[derive(Debug, Deserialize, IntoParams, Default, serde::Serialize, ToSchema)]
pub struct PaginatedTokenPickGroupResponse {
    /// Group name and token picks
    pub items: HashMap<String, Vec<TokenPickResponse>>,
    pub total: i64,
    pub page: u32,
    pub limit: u32,
    pub total_pages: u32,
}

#[derive(Debug, Deserialize, IntoParams, Default)]
pub struct TokenGroupQuery {
    #[serde(deserialize_with = "crate::utils::serde_utils::deserialize_optional_uuid")]
    pub user_id: Option<Uuid>,
    #[param(default = 1)]
    pub page: u32,
    #[param(default = 10)]
    pub limit: u32,
    pub order_by: Option<PickLeaderboardSort>,
    pub order_direction: Option<String>,
    #[param(default = false)]
    pub get_all: Option<bool>,
    pub group_ids: Option<Vec<i64>>,
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RemoveUserRequest {
    pub user_id: Uuid,
    pub group_id: i64,
}
