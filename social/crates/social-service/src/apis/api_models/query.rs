use crate::utils::time::{default_time_period, TimePeriod};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, ToSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProfileLeaderboardSort {
    #[default]
    PickReturns,
    HitRate,
    RealizedProfit,
    TotalPicks,
    MostRecentPick,
    AverageReturn,
    GreatestHits,
}

#[derive(Debug, Deserialize, IntoParams, Default)]
pub struct TokenQuery {
    pub username: Option<String>,
    pub picked_after: Option<TimePeriod>,
    #[param(default = 1)]
    pub page: u32,
    #[param(default = 10)]
    pub limit: u32,
    pub order_by: Option<PickLeaderboardSort>,
    pub order_direction: Option<String>,
    #[param(default = false)]
    pub get_all: Option<bool>,
    pub group_ids: Option<Vec<i64>>,
    #[param(default = false)]
    pub following: Option<bool>,
}

#[derive(Debug, Deserialize, ToSchema, Default)]
pub struct ProfileQuery {
    pub username: String,
    #[serde(default = "default_time_period")]
    pub picked_after: TimePeriod,
    pub group_id: Option<i64>,
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
    pub order_by: Option<ProfileLeaderboardSort>,
    pub order_direction: Option<String>,
}

#[derive(Debug, Deserialize, IntoParams, Default)]
pub struct GroupMembersQuery {
    pub username: Option<String>,
    #[param(default = 1)]
    pub page: u32,
    #[param(default = 10)]
    pub limit: u32,
    pub order_by: Option<ProfileLeaderboardSort>,
    pub order_direction: Option<String>,
}

#[derive(Deserialize, Serialize, ToSchema, IntoParams, Debug, Default)]
pub struct ProfileLeaderboardQuery {
    #[serde(default)]
    pub sort: Option<ProfileLeaderboardSort>,
    #[serde(default)]
    pub order: Option<String>,
    #[serde(default = "default_time_period")]
    pub picked_after: TimePeriod,
    pub group_id: Option<i64>,
    #[serde(default)]
    pub following: bool,
    pub username: Option<String>,
    pub user_id: Option<Uuid>,
}

#[derive(Deserialize, IntoParams, Default)]
pub struct ListGroupMembersQuery {
    pub sort: Option<ProfileLeaderboardSort>,
    pub user_id: Uuid,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, ToSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum PickLeaderboardSort {
    Hottest,
    Newest,
    #[default]
    Reached,
}

impl ToString for PickLeaderboardSort {
    fn to_string(&self) -> String {
        match self {
            PickLeaderboardSort::Hottest => "t.volume_24h".to_string(),
            PickLeaderboardSort::Newest => "call_date".to_string(),
            PickLeaderboardSort::Reached => "highest_multiplier".to_string(),
        }
    }
}
