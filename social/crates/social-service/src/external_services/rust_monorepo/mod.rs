use std::collections::HashMap;

use get_latest_w_metadata::LatestTokenMetadataResponse;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use reqwest::Client;
use tracing::{debug, error, warn};

use crate::utils::errors::app_error::AppError;

pub mod get_latest_w_metadata;

pub struct RustMonorepoService {
    client: Client,
    rust_monorepo_url: String,
    api_key: String,
}

impl RustMonorepoService {
    pub fn new(rust_monorepo_url: String, api_key: String) -> Self {
        let client = Client::new();
        Self {
            client,
            rust_monorepo_url,
            api_key,
        }
    }

    pub async fn get_latest_w_metadata(
        &self,
        addresses: &[String],
    ) -> Result<HashMap<String, LatestTokenMetadataResponse>, AppError> {
        let body = serde_json::to_string(&addresses).map_err(|e| {
            warn!("Failed to serialize addresses: {}", e);
            AppError::InternalServerError()
        })?;
        debug!("Sending data to rust monorepo: {}", body.len());
        let url = format!("{}/price/latest-with-metadata", self.rust_monorepo_url);
        let res = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("X-API-Key", self.api_key.clone())
            .body(body)
            .send()
            .await?;

        let tokens: Vec<LatestTokenMetadataResponse> = match res.json().await {
            Ok(t) => t,
            Err(e) => {
                error!("Failed to deserialize response for {:?}: {}", addresses, e);
                return Err(AppError::RequestError(e));
            }
        };

        let result = tokens
            .into_par_iter()
            .map(|r| (r.address.clone(), r))
            .collect();

        Ok(result)
    }
}
