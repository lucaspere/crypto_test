use redis::{aio::MultiplexedConnection, Client, RedisError};
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
        let serialized = serde_json::to_string(value)?;
        let mut connection = self.connection.clone();
        let _: () = redis::cmd("SET")
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
        let _: () = redis::cmd("DEL")
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
            let _: () = redis::cmd("DEL")
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
        let result: Option<String> = redis::cmd("SET")
            .arg(key)
            .arg(value)
            .arg("NX")
            .arg("EX")
            .arg(ttl_seconds)
            .query_async(&mut connection)
            .await?;

        Ok(result.is_some())
    }

    pub async fn execute_pipe(&self, pipe: redis::Pipeline) -> Result<(), RedisError> {
        let mut connection = self.connection.clone();
        let _: () = pipe.query_async(&mut connection).await?;
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
