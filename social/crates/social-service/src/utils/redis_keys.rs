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
}
impl RedisKeys {
    // Time-based leaderboard prefixes
    pub const LEADERBOARD_24H: &'static str = "leaderboard:24h";
    pub const LEADERBOARD_7D: &'static str = "leaderboard:7d";
    pub const LEADERBOARD_1Y: &'static str = "leaderboard:1y";

    // Leaderboard metrics
    pub const METRIC_RETURNS: &'static str = ":returns";
    pub const METRIC_HIT_RATE: &'static str = ":hit_rate";
    pub const METRIC_TOTAL_PICKS: &'static str = ":total_picks";

    pub fn get_leaderboard_key(timeframe: &str, metric: &str) -> String {
        format!("{}{}", timeframe, metric)
    }
}
