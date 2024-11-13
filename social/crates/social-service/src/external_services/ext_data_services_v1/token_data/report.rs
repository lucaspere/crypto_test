use super::{types::TokenReportResponse, TokenDataService};
use crate::utils::api_errors::ApiError;

impl TokenDataService {
    pub async fn get_token_report(
        &self,
        contract_addresses: &[String],
    ) -> Result<TokenReportResponse, ApiError> {
        let cache_key = format!("token_report:{}", contract_addresses.join(","));
        if let Some(cached_response) = self
            .redis_service
            .get_cached::<TokenReportResponse>(&cache_key)
            .await?
        {
            return Ok(cached_response);
        }

        let address_params: Vec<String> = contract_addresses
            .iter()
            .map(|address| format!("addresses={}", urlencoding::encode(address)))
            .collect();

        let url = format!(
            "{}/token_data/report?{}",
            self.base_url(),
            address_params.join("&")
        );

        let response = self
            .client()
            .get(&url)
            .send()
            .await?
            .json::<TokenReportResponse>()
            .await?;

        self.redis_service
            .set_cached(&cache_key, &response, 3600)
            .await?;

        Ok(response)
    }
}
