use std::collections::HashMap;

use get_latest_w_metadata::LatestTokenMetadataResponse;
use reqwest::Client;
use tracing::info;

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
        let res = self
            .client
            .post(format!(
                "{}/price/latest-with-metadata",
                self.rust_monorepo_url,
            ))
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await?;

        let res = res
            .json::<Vec<LatestTokenMetadataResponse>>()
            .await?
            .into_iter()
            .map(|r| (r.address.clone(), r))
            .collect();

        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn should_get_latest_w_metadata() {
        let service = RustMonorepoService::new("http://localhost:698".to_string());
        let res = service
            .get_latest_w_metadata(vec![
                "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN".to_string()
            ])
            .await;

        assert!(res.is_ok());
        assert_eq!(res.unwrap().len(), 1);
    }
}
