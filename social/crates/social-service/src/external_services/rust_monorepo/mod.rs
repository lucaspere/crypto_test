use std::collections::HashMap;

use get_latest_w_metadata::LatestTokenMetadataResponse;
use reqwest::Client;
use tracing::{error, info};

use crate::utils::api_errors::ApiError;

pub mod get_latest_w_metadata;

pub struct RustMonorepoService {
    client: Client,
    rust_monorepo_url: String,
}

impl RustMonorepoService {
    pub fn new(rust_monorepo_url: String) -> Self {
        let client = Client::new();
        Self {
            client,
            rust_monorepo_url,
        }
    }

    pub async fn get_latest_w_metadata(
        &self,
        addresses: Vec<String>,
    ) -> Result<HashMap<String, LatestTokenMetadataResponse>, ApiError> {
        let body = serde_json::to_string(&addresses)
            .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
        info!("Sending data to rust monorepo: {:?}", body);
        let url = format!("{}/price/latest-with-metadata", self.rust_monorepo_url);
        let res = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await?;

        let tokens: Vec<LatestTokenMetadataResponse> = match res.json().await {
            Ok(t) => t,
            Err(e) => {
                error!("Failed to deserialize response for {:?}: {}", addresses, e);
                return Err(ApiError::RequestError(e));
            }
        };

        let result = tokens.into_iter().map(|r| (r.address.clone(), r)).collect();

        Ok(result)
    }
}
