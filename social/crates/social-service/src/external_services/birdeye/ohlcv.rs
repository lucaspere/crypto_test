use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::utils::errors::app_error::AppError;

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

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct OHLCVResponse {
    pub success: bool,
    pub data: Option<BirdeyeOHLCVItems>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
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
    ) -> Result<BirdeyeOHLCVItem, AppError> {
        let query = BirdeyeOHLCVQuery {
            address: address.to_owned(),
            interval: resolution.to_owned(),
            time_from: time_start,
            time_to: time_end,
        };

        info!("Fetching OHLCV for address: {}, chain: {}, time_start: {}, time_end: {}, resolution: {}", address, chain, time_start, time_end, resolution);
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
            error!("Error fetching OHLCV {}: {}", json_str, e);
            AppError::InternalServerError()
        })?;

        let max_high = ohlcv_response
            .data
            .unwrap_or_default()
            .items
            .into_iter()
            .max_by_key(|item| item.high)
            .unwrap_or_default();

        Ok(max_high)
    }
}
