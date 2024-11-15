use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Duration, Utc};

#[inline]
pub fn get_current_time() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

pub enum TimePeriod {
    Day,
    Week,
    Month,
}

impl TimePeriod {
    pub fn seconds(&self) -> i64 {
        match self {
            TimePeriod::Day => 86400,
            TimePeriod::Week => 604800,
            TimePeriod::Month => 2629746,
        }
    }

    pub fn datetime(&self) -> DateTime<Utc> {
        Utc::now() - Duration::seconds(self.seconds())
    }
}
