use std::{
    fmt::Display,
    time::{SystemTime, UNIX_EPOCH},
};

use chrono::{DateTime, Duration, FixedOffset, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[inline]
pub fn get_current_time() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

pub fn default_time_period() -> TimePeriod {
    TimePeriod::Month
}

#[derive(Deserialize, ToSchema, Debug, Clone, Serialize, Default)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum TimePeriod {
    SixHours,
    Day,
    Week,
    #[default]
    Month,
    AllTime,
}

impl TimePeriod {
    pub fn seconds(&self) -> i64 {
        match self {
            TimePeriod::SixHours => 21600,
            TimePeriod::Day => 86400,
            TimePeriod::Week => 604800,
            TimePeriod::Month => 2629746,
            TimePeriod::AllTime => 31556952,
        }
    }

    pub fn get_start_datetime(&self) -> DateTime<Utc> {
        Utc::now() - Duration::seconds(self.seconds())
    }

    pub fn to_date_time(&self, now: DateTime<FixedOffset>) -> DateTime<FixedOffset> {
        match self {
            TimePeriod::SixHours => now - Duration::hours(6),
            TimePeriod::Day => now - Duration::days(1),
            TimePeriod::Week => now - Duration::weeks(1),
            TimePeriod::Month => now - Duration::days(30),
            TimePeriod::AllTime => now - Duration::days(365),
        }
    }
}

impl Display for TimePeriod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimePeriod::SixHours => write!(f, "six_hours"),
            TimePeriod::Day => write!(f, "day"),
            TimePeriod::Week => write!(f, "week"),
            TimePeriod::Month => write!(f, "month"),
            TimePeriod::AllTime => write!(f, "all_time"),
        }
    }
}
