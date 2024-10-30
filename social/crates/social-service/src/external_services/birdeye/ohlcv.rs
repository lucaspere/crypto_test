use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::utils::api_errors::ApiError;

use super::BirdeyeService;

pub const BIRDEYE_OHLCV_URL: &str = "https://public-api.birdeye.so/defi/ohlcv";

#[derive(Serialize)]
pub struct BirdeyeOHLCVQuery {
    pub address: String,
    #[serde(rename = "type")]
    pub interval: String,
    pub time_from: i64,
    pub time_to: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OHLCVResponse {
    pub success: bool,
    pub data: BirdeyeOHLCVItems,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BirdeyeOHLCVItems {
    pub items: Vec<BirdeyeOHLCVItem>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct BirdeyeOHLCVItem {
    #[serde(rename = "o")]
    pub open: Decimal,
    #[serde(rename = "h")]
    pub high: Decimal,
    #[serde(rename = "l")]
    pub low: Decimal,
    #[serde(rename = "c")]
    pub close: Decimal,
    #[serde(rename = "v")]
    pub volume: Decimal,
    #[serde(rename = "unixTime")]
    pub unix_time: i64,
    pub address: String,
    #[serde(rename = "type")]
    pub interval: String,
}

impl BirdeyeService {
    pub async fn get_ohlcv_request(
        &self,
        chain: &str,
        address: &str,
        time_start: i64,
        time_end: i64,
        resolution: &str,
    ) -> Result<BirdeyeOHLCVItem, ApiError> {
        let query = BirdeyeOHLCVQuery {
            address: address.to_owned(),
            interval: resolution.to_owned(),
            time_from: time_start,
            time_to: time_end,
        };

        let response = self
            .client
            .get(BIRDEYE_OHLCV_URL)
            .header("X-API-KEY", &self.api_key)
            .header("x-chain", chain)
            .query(&query)
            .send()
            .await?;

        let json_str = response.text().await?;

        let ohlcv_response: OHLCVResponse = serde_json::from_str(&json_str).map_err(|e| {
            error!("Error fetching OHLCV: {}", e);
            ApiError::InternalServerError("Internal server error".to_string())
        })?;

        let max_high = ohlcv_response
            .data
            .items
            .into_iter()
            .max_by_key(|item| item.high)
            .unwrap_or_default();

        Ok(max_high)
    }
}
