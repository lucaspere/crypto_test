use std::time::{SystemTime, UNIX_EPOCH};

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

impl ToString for TimePeriod {
    fn to_string(&self) -> String {
        match self {
            TimePeriod::SixHours => "six_hours".to_string(),
            TimePeriod::Day => "day".to_string(),
            TimePeriod::Week => "week".to_string(),
            TimePeriod::Month => "month".to_string(),
            TimePeriod::AllTime => "all_time".to_string(),
        }
    }
}
