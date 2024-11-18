use chrono::{DateTime, Utc};
use redis::RedisError;
use rust_decimal::{prelude::ToPrimitive, Decimal};
use serde::{Deserialize, Serialize};

use crate::{models::token_picks::TokenPickResponse, utils::redis_keys::RedisKeys};

use super::redis_service::RedisService;

#[derive(Serialize, Deserialize)]
pub struct ProfileCacheStats {
    pub total_picks: i64,
    pub hits_2x: i64,
    pub total_returns: Decimal,
    pub hit_rate: f64,
    pub best_pick_multiplier: Decimal,
    pub best_pick_symbol: String,
    pub last_updated: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
pub struct TokenPickStats {
    pub total_picks: i64,
    pub hits_2x: i64,
    pub highest_multiplier: Decimal,
    pub current_price: Decimal,
    pub last_updated: DateTime<Utc>,
}

impl RedisService {
    pub async fn update_leaderboards(
        &self,
        username: &str,
        stats: &ProfileCacheStats,
    ) -> Result<(), RedisError> {
        let mut pipe = redis::pipe();

        // Update pick returns leaderboard
        pipe.zadd(
            RedisKeys::PICK_RETURNS_LEADERBOARD,
            username,
            stats.total_returns.to_f64().unwrap_or_default(),
        );

        // Update hit rate leaderboard
        pipe.zadd(RedisKeys::HIT_RATE_LEADERBOARD, username, stats.hit_rate);

        // Update total picks leaderboard
        pipe.zadd(
            RedisKeys::TOTAL_PICKS_LEADERBOARD,
            username,
            stats.total_picks,
        );

        pipe.expire(
            RedisKeys::PICK_RETURNS_LEADERBOARD,
            RedisKeys::CACHE_TTL as i64,
        );
        pipe.expire(RedisKeys::HIT_RATE_LEADERBOARD, RedisKeys::CACHE_TTL as i64);
        pipe.expire(
            RedisKeys::TOTAL_PICKS_LEADERBOARD,
            RedisKeys::CACHE_TTL as i64,
        );

        self.execute_pipe(pipe).await?;
        Ok(())
    }

    pub async fn cache_profile_stats(
        &self,
        username: &str,
        stats: &ProfileCacheStats,
    ) -> Result<(), RedisError> {
        let key = RedisKeys::get_profile_stats_key(username);
        self.set_cached(&key, stats, RedisKeys::CACHE_TTL).await
    }

    pub async fn cache_token_stats(
        &self,
        address: &str,
        stats: &TokenPickResponse,
    ) -> Result<(), RedisError> {
        let key = RedisKeys::get_token_stats_key(address);
        self.set_cached(&key, stats, RedisKeys::CACHE_TTL).await
    }
}
