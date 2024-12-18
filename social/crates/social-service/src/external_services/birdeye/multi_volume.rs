use std::collections::HashMap;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

use crate::utils::errors::app_error::AppError;

use super::BirdeyeService;

pub const BIRDEYE_OHLCV_URL: &str = "https://public-api.birdeye.so/defi/price_volume/multi";

#[derive(Serialize, Debug)]
pub struct BirdeyeMultiVolumeBody {
    #[serde(rename = "type")]
    pub timeframe: String,
    pub list_address: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct BirdeyeMultiVolumeResponse {
    pub success: bool,
    pub data: Option<BirdeyeMultiVolumeItems>,
}

pub type BirdeyeMultiVolumeItems = HashMap<String, Option<BirdeyeMultiVolumeItem>>;

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct BirdeyeMultiVolumeItem {
    pub price: Decimal,
    pub update_unix_time: i64,
    pub update_human_time: String,
    #[serde(rename = "volumeUSD")]
    pub volume_usd: Decimal,
    pub volume_change_percent: Decimal,
    pub price_change_percent: Decimal,
}

impl BirdeyeService {
    pub async fn get_multi_volume_request(
        &self,
        chain: &str,
        body: BirdeyeMultiVolumeBody,
    ) -> Result<BirdeyeMultiVolumeResponse, AppError> {
        debug!(
            "Fetching multi volume for addresses: {:?}, chain: {}, timeframe: {}",
            body.list_address, chain, body.timeframe
        );
        let response = self
            .client
            .post(BIRDEYE_OHLCV_URL)
            .header("X-API-KEY", &self.api_key)
            .header("x-chain", chain)
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                error!("Error fetching multi volume: {}", e);
                AppError::InternalServerError()
            })?;

        let multi_volume_response: BirdeyeMultiVolumeResponse =
            response.json().await.map_err(|e| {
                error!(
                    "Error deserializing multi volume: {} with body: {:?}",
                    e, body
                );
                AppError::InternalServerError()
            })?;

        Ok(multi_volume_response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_multi_volume_request() {
        let birdeye_service = BirdeyeMultiVolumeBody {
            timeframe: "24h".to_string(),
            list_address: vec![
                "eL5fUxj2J4CiQsmW85k5FG9DvuQjjUoBHoQBi2Kpump".to_string(),
                "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263".to_string(),
            ]
            .join(","),
        };
        let response = serde_json::to_string(&birdeye_service).unwrap();
        println!(" Response {:#?}", response);
    }
}
