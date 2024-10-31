use redis::{aio::MultiplexedConnection, AsyncCommands, Client, RedisError};
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct RedisService {
    connection: Arc<Mutex<MultiplexedConnection>>,
}

impl RedisService {
    pub async fn new(redis_url: &str) -> Result<Self, RedisError> {
        let client = Client::open(redis_url)?;
        let connection = client.get_multiplexed_async_connection().await?;

        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
        })
    }

    pub async fn get_cached<T: DeserializeOwned>(
        &self,
        key: &str,
    ) -> Result<Option<T>, RedisError> {
        let mut conn = self.connection.lock().await;
        let result: Option<String> = conn.get(key).await?;

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
        let mut conn = self.connection.lock().await;
        let serialized = serde_json::to_string(value).map_err(|e| {
            RedisError::from((
                redis::ErrorKind::InvalidClientConfig,
                "Serialization error",
                e.to_string(),
            ))
        })?;

        conn.set_ex(key, serialized, ttl_seconds).await
    }

    pub async fn delete_cached(&self, key: &str) -> Result<(), RedisError> {
        let mut conn = self.connection.lock().await;
        conn.del(key).await
    }
}
