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
}
