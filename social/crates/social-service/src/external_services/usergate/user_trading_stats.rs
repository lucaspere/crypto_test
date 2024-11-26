use rust_decimal::Decimal;
use serde::Deserialize;

use crate::utils::api_errors::ApiError;

use super::UserGateService;

#[derive(Deserialize, Default)]
pub struct UserTradingStatsResponse {
    pub trade_count: u64,
    pub trading_volume_usd: Decimal,
}

impl UserGateService {
    pub async fn get_user_trading_stats(
        &self,
        user_id: &str,
    ) -> Result<UserTradingStatsResponse, ApiError> {
        let url = format!("{}/api/user/{}/trading-stats", self.base_url, user_id);
        let response = self.client.get(url).send().await?;
        response.json().await.map_err(ApiError::from)
    }
}
