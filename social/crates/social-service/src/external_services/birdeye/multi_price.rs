use std::collections::HashMap;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

use crate::utils::errors::app_error::AppError;

use super::BirdeyeService;

pub const BIRDEYE_OHLCV_URL: &str = "https://public-api.birdeye.so/defi/multi_price";

#[derive(Serialize, Debug)]
pub struct BirdeyeMultiPriceQuery {
    pub check_liquidity: Option<i32>,
    pub include_liquidity: bool,
    pub list_address: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct BirdeyeMultiPriceResponse {
    pub success: bool,
    pub data: Option<BirdeyeMultiPriceItems>,
}

pub type BirdeyeMultiPriceItems = HashMap<String, Option<BirdeyeMultiPriceItem>>;

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct BirdeyeMultiPriceItem {
    pub value: Decimal,
    pub update_unix_time: i64,
    pub update_human_time: String,
    pub price_change_24h: Option<Decimal>,
    pub liquidity: Option<Decimal>,
}

impl BirdeyeService {
    pub async fn get_multi_price_request(
        &self,
        chain: &str,
        query: BirdeyeMultiPriceQuery,
    ) -> Result<BirdeyeMultiPriceResponse, AppError> {
        debug!(
            "Fetching multi price for addresses: {:?}, chain: {}, check_liquidity: {:?}, include_liquidity: {}",
            query.list_address, chain, query.check_liquidity, query.include_liquidity
        );
        let response = self
            .client
            .get(BIRDEYE_OHLCV_URL)
            .header("X-API-KEY", &self.api_key)
            .header("x-chain", chain)
            .query(&query)
            .send()
            .await
            .map_err(|e| {
                error!("Error fetching multi price: {}", e);
                AppError::InternalServerError()
            })?;

        let multi_price_response: BirdeyeMultiPriceResponse =
            response.json().await.map_err(|e| {
                error!("Error text multi price: {} with query: {:?}", e, query);
                AppError::InternalServerError()
            })?;

        Ok(multi_price_response)
    }
}
