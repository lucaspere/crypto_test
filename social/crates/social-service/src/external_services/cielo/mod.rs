pub mod aggregated_wallet;

use std::sync::Arc;

use reqwest::{header, Client};

use crate::services::redis_service::RedisService;

pub struct CieloService {
    client: Client,
    api_key: String,
    base_url: String,
    redis_service: Arc<RedisService>,
}

impl CieloService {
    pub fn new(api_key: String, redis_service: Arc<RedisService>) -> Self {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "X-API-Key",
            header::HeaderValue::from_str(&api_key).unwrap(),
        );

        let client = Client::builder().default_headers(headers).build().unwrap();

        Self {
            client,
            api_key,
            base_url: "https://feed-api.cielo.finance/api/v1".to_string(),
            redis_service,
        }
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}
