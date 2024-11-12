use crate::apis::profile_handlers::TimeRange;
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

#[derive(Debug, Deserialize, IntoParams, Default)]
pub struct TokenQuery {
    pub username: Option<String>,
    pub picked_after: Option<TimeRange>,
    #[param(default = 1)]
    pub page: u32,
    #[param(default = 10)]
    pub limit: u32,
    pub order_by: Option<String>,
    pub order_direction: Option<String>,
    #[param(default = false)]
    pub get_all: Option<bool>,
    pub group_ids: Option<Vec<i64>>,
}

#[derive(Debug, Deserialize, ToSchema, Default)]
pub struct ProfileQuery {
    pub username: String,
    #[serde(default = "default_time_range")]
    pub picked_after: TimeRange,
    pub group_id: Option<i64>,
}

fn default_time_range() -> TimeRange {
    TimeRange::Year
}

#[derive(Debug, Deserialize, IntoParams, Default)]
pub struct ListGroupsQuery {
    pub user_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, IntoParams, Default)]
pub struct GroupPicksQuery {
    pub username: Option<String>,
    #[param(default = 1)]
    pub page: u32,
    #[param(default = 10)]
    pub limit: u32,
    pub order_by: Option<String>,
    pub order_direction: Option<String>,
}

#[derive(Debug, Deserialize, IntoParams, Default)]
pub struct GroupMembersQuery {
    pub username: Option<String>,
    #[param(default = 1)]
    pub page: u32,
    #[param(default = 10)]
    pub limit: u32,
    pub order_by: Option<String>,
    pub order_direction: Option<String>,
}
