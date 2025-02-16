use super::time::TimePeriod;

pub struct RedisKeys;

impl RedisKeys {
    // Leaderboard keys
    pub const PICK_RETURNS_LEADERBOARD: &'static str = "leaderboard:pick_returns";
    pub const HIT_RATE_LEADERBOARD: &'static str = "leaderboard:hit_rate";
    pub const TOTAL_PICKS_LEADERBOARD: &'static str = "leaderboard:total_picks";

    // Profile stats keys
    pub const PROFILE_STATS_PREFIX: &'static str = "profile:stats:";
    pub const TOKEN_PICK_STATS_PREFIX: &'static str = "token:stats:";

    // Cache TTL (24 hours)
    pub const CACHE_TTL: u64 = 86400;

    // Environment prefix
    pub const PROD_ENV: &'static str = "prod";
    pub const STAGING_ENV: &'static str = "staging";

    pub fn get_env_prefix() -> &'static str {
        match std::env::var("ENVIRONMENT")
            .unwrap_or_else(|_| "prod".to_string())
            .as_str()
        {
            "staging" => Self::STAGING_ENV,
            _ => Self::PROD_ENV,
        }
    }

    pub fn get_profile_stats_key(username: &str) -> String {
        format!(
            "{}:{}{}",
            Self::get_env_prefix(),
            Self::PROFILE_STATS_PREFIX,
            username
        )
    }

    pub fn get_token_stats_key(address: &str) -> String {
        format!(
            "{}:{}{}",
            Self::get_env_prefix(),
            Self::TOKEN_PICK_STATS_PREFIX,
            address
        )
    }

    pub fn get_ttl_for_timeframe(timeframe: &TimePeriod) -> i64 {
        timeframe.seconds() / 2
    }
}

impl RedisKeys {
    // Leaderboard metrics
    pub const METRIC_RETURNS: &'static str = ":returns";
    pub const METRIC_HIT_RATE: &'static str = ":hit_rate";
    pub const METRIC_TOTAL_PICKS: &'static str = ":total_picks";

    pub fn get_leaderboard_key(timeframe: &str, metric: &str) -> String {
        format!(
            "{}:{}",
            Self::get_env_prefix(),
            format!("{}{}", timeframe, metric)
        )
    }
}

impl RedisKeys {
    // Add group leaderboard keys
    pub const PROCESSING_LOCK_KEY: &str = "token_picks:processing_lock";
    pub const NOTIFY_FOLLOWERS_LOCK_KEY: &str = "token_picks:notify_followers_lock:";
    pub const GROUP_LEADERBOARD_PREFIX: &'static str = "group:leaderboard";

    pub fn get_group_leaderboard_key(group_id: i64, timeframe: &str) -> String {
        format!(
            "{}:{}:{}:{}",
            Self::get_env_prefix(),
            Self::GROUP_LEADERBOARD_PREFIX,
            group_id,
            timeframe
        )
    }

    pub fn get_group_leaderboard_data_key(group_id: i64, timeframe: &str) -> String {
        format!(
            "{}:{}:{}:{}:data",
            Self::get_env_prefix(),
            Self::GROUP_LEADERBOARD_PREFIX,
            group_id,
            timeframe
        )
    }
}
