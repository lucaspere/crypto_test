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

    pub fn get_profile_stats_key(username: &str) -> String {
        format!("{}{}", Self::PROFILE_STATS_PREFIX, username)
    }

    pub fn get_token_stats_key(address: &str) -> String {
        format!("{}{}", Self::TOKEN_PICK_STATS_PREFIX, address)
    }

    pub fn get_ttl_for_timeframe(timeframe: &str) -> i64 {
        match timeframe {
            Self::LEADERBOARD_24H => 86400,
            Self::LEADERBOARD_7D => 604800,
            Self::LEADERBOARD_1Y => 31536000,
            _ => 3600,
        }
    }
}
impl RedisKeys {
    // Time-based leaderboard prefixes
    pub const LEADERBOARD_24H: &'static str = "24h";
    pub const LEADERBOARD_7D: &'static str = "7d";
    pub const LEADERBOARD_1Y: &'static str = "1y";

    // Leaderboard metrics
    pub const METRIC_RETURNS: &'static str = ":returns";
    pub const METRIC_HIT_RATE: &'static str = ":hit_rate";
    pub const METRIC_TOTAL_PICKS: &'static str = ":total_picks";

    pub fn get_leaderboard_key(timeframe: &str, metric: &str) -> String {
        format!("{}{}", timeframe, metric)
    }
}

impl RedisKeys {
    // Add group leaderboard keys
    pub const GROUP_LEADERBOARD_PREFIX: &'static str = "group:leaderboard";

    pub fn get_group_leaderboard_key(group_id: i64, timeframe: &str) -> String {
        format!(
            "{}:{}:{}",
            Self::GROUP_LEADERBOARD_PREFIX,
            group_id,
            timeframe
        )
    }

    pub fn get_group_leaderboard_data_key(group_id: i64, timeframe: &str) -> String {
        format!(
            "{}:{}:{}:data",
            Self::GROUP_LEADERBOARD_PREFIX,
            group_id,
            timeframe
        )
    }
}
