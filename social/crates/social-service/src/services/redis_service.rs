use redis::{aio::MultiplexedConnection, Client, Pipeline, RedisError};
use serde::{de::DeserializeOwned, Serialize};

pub struct RedisService {
    connection: MultiplexedConnection,
}

impl RedisService {
    pub async fn new(redis_url: &str) -> Result<Self, RedisError> {
        let client = Client::open(redis_url)?;
        let connection = client.get_multiplexed_async_connection().await?;

        Ok(Self { connection })
    }

    pub async fn get_cached<T: DeserializeOwned>(
        &self,
        key: &str,
    ) -> Result<Option<T>, RedisError> {
        let mut connection = self.connection.clone();
        let result: Option<String> = redis::cmd("GET")
            .arg(key)
            .query_async(&mut connection)
            .await?;

        match result {
            Some(data) => Ok(serde_json::from_str(&data).ok()),
            None => Ok(None),
        }
    }

    pub async fn set_cached<T: Serialize>(
        &self,
        key: &str,
        value: &T,
        ttl_seconds: u64,
    ) -> Result<(), RedisError> {
        let serialized = serde_json::to_string(value).map_err(|e| {
            RedisError::from((
                redis::ErrorKind::InvalidClientConfig,
                "Serialization error",
                e.to_string(),
            ))
        })?;
        let mut connection = self.connection.clone();
        redis::cmd("SET")
            .arg(key)
            .arg(&serialized)
            .arg("EX")
            .arg(ttl_seconds)
            .query_async(&mut connection)
            .await?;
        Ok(())
    }

    pub async fn delete_cached(&self, key: &str) -> Result<(), RedisError> {
        let mut connection = self.connection.clone();
        redis::cmd("DEL")
            .arg(key)
            .query_async(&mut connection)
            .await?;
        Ok(())
    }

    pub async fn delete_pattern(&self, pattern: &str) -> Result<(), RedisError> {
        let mut connection = self.connection.clone();
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(pattern)
            .query_async(&mut connection)
            .await?;

        if !keys.is_empty() {
            redis::cmd("DEL")
                .arg(keys)
                .query_async(&mut connection)
                .await?;
        }
        Ok(())
    }

    pub async fn set_nx(
        &self,
        key: &str,
        value: &str,
        ttl_seconds: u64,
    ) -> Result<bool, RedisError> {
        let mut connection = self.connection.clone();
        let result: i32 = redis::cmd("SET")
            .arg(key)
            .arg(value)
            .arg("NX")
            .arg("EX")
            .arg(ttl_seconds)
            .query_async(&mut connection)
            .await?;

        Ok(result == 1)
    }

    pub async fn execute_pipe(&self, pipe: Pipeline) -> Result<(), RedisError> {
        let mut connection = self.connection.clone();
        pipe.query_async(&mut connection).await?;
        Ok(())
    }

    pub async fn add_to_leaderboard(
        &self,
        leaderboard_key: &str,
        member: &str,
        score: f64,
    ) -> Result<(), RedisError> {
        let mut connection = self.connection.clone();
        redis::cmd("ZADD")
            .arg(leaderboard_key)
            .arg(score)
            .arg(member)
            .query_async(&mut connection)
            .await?;
        Ok(())
    }

    pub async fn get_leaderboard(
        &self,
        leaderboard_key: &str,
        start: isize,
        stop: isize,
    ) -> Result<Vec<(String, f64)>, RedisError> {
        let mut connection = self.connection.clone();
        redis::cmd("ZREVRANGE")
            .arg(leaderboard_key)
            .arg(start)
            .arg(stop)
            .arg("WITHSCORES")
            .query_async(&mut connection)
            .await
    }

    pub async fn update_group_leaderboard(
        &self,
        group_id: i64,
        token_pick_id: i64,
        multiplier: f64,
    ) -> Result<(), RedisError> {
        let leaderboard_key = format!("leaderboard:group:{}", group_id);
        self.add_to_leaderboard(&leaderboard_key, &token_pick_id.to_string(), multiplier)
            .await?;
        // Set TTL to 1 hour to periodically refresh from DB
        let mut connection = self.connection.clone();
        redis::cmd("EXPIRE")
            .arg(&leaderboard_key)
            .arg(3600)
            .query_async(&mut connection)
            .await?;
        Ok(())
    }

    pub async fn zrange_by_score(
        &self,
        key: &str,
        start: isize,
        stop: isize,
    ) -> Result<Vec<String>, RedisError> {
        let mut connection = self.connection.clone();
        redis::cmd("ZREVRANGE")
            .arg(key)
            .arg(start)
            .arg(stop)
            .query_async(&mut connection)
            .await
    }

    pub async fn hget_multiple<T: DeserializeOwned>(
        &self,
        key: &str,
        fields: &[String],
    ) -> Result<Option<Vec<T>>, RedisError> {
        let mut connection = self.connection.clone();
        let values: Vec<Option<String>> = redis::cmd("HMGET")
            .arg(key)
            .arg(fields)
            .query_async(&mut connection)
            .await?;

        let results: Vec<T> = values
            .into_iter()
            .filter_map(|v| v.and_then(|s| serde_json::from_str(&s).ok()))
            .collect();

        if results.is_empty() {
            Ok(None)
        } else {
            Ok(Some(results))
        }
    }
}
