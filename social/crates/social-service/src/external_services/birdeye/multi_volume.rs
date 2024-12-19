use std::collections::HashMap;

use crate::utils::serde_utils::deserialize_null_as_zero;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

use crate::utils::errors::app_error::AppError;

use super::BirdeyeService;

pub const BIRDEYE_OHLCV_URL: &str = "https://public-api.birdeye.so/defi/price_volume/multi";

#[derive(Serialize, Debug, Deserialize)]
pub struct BirdeyeMultiVolumeBody {
    #[serde(rename = "type")]
    pub timeframe: Option<String>,
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
    #[serde(rename = "volumeUSD", deserialize_with = "deserialize_null_as_zero")]
    pub volume_usd: Decimal,
    pub volume_change_percent: Option<Decimal>,
    pub price_change_percent: Option<Decimal>,
}

impl BirdeyeService {
    pub async fn get_multi_volume_request(
        &self,
        chain: &str,
        body: BirdeyeMultiVolumeBody,
    ) -> Result<BirdeyeMultiVolumeResponse, AppError> {
        debug!(
            "Fetching multi volume for addresses: {:?}, chain: {}, timeframe: {:?}",
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

        let response = response.text().await.map_err(|e| {
            error!("Error fetching multi volume: {}", e);
            AppError::InternalServerError()
        })?;
        let multi_volume_response: BirdeyeMultiVolumeResponse = serde_json::from_str(&response)
            .map_err(|_| {
                error!(
                    "Error deserializing multi volume with response: {}",
                    response
                );
                AppError::InternalServerError()
            })?;

        Ok(multi_volume_response)
    }
}
