use std::fmt::Display;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, ToSchema, Default)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "sort")]
pub enum ProfileSort {
    #[default]
    PickReturns,
    HitRate,
    RealizedProfit,
    TotalPicks,
    MostRecentPick,
    AverageReturn,
    GreatestHits,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, ToSchema, Default)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "sort")]
pub enum PickSort {
    Hottest,
    #[serde(rename = "call_date")]
    Newest,
    #[default]
    #[serde(rename = "highest_multiplier")]
    HighestReturn,
}

impl Display for PickSort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

#[derive(Debug, Deserialize, Serialize, ToSchema, Default)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "direction")]
pub enum SortDirection {
    Asc,
    #[default]
    Desc,
}

impl Display for SortDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}
