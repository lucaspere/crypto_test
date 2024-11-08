use super::CieloService;
use crate::{models::tokens::Chain, utils::api_errors::ApiError};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AggregatedWalletStatsResponse {
    pub status: String,
    pub data: AggregatedWalletStats,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AggregatedWalletStats {
    pub wallet: String,
    pub realized_pnl_usd: Decimal,
    pub realized_roi_percentage: Decimal,
    pub tokens_traded: i32,
    pub unrealized_pnl_usd: Decimal,
    pub unrealized_roi_percentage: Decimal,
    pub winrate: Decimal,
    pub average_holding_time: Decimal,
    pub combined_pnl_usd: Decimal,
    pub combined_roi_percentage: Decimal,
}

pub struct AggregatedWalletStatsQuery {
    pub timeframe: Option<String>,
    pub cex_transfer: Option<bool>,
    pub chain: Chain,
}

impl CieloService {
    pub async fn get_wallet_stats(
        &self,
        wallet_address: &str,
        query: Option<AggregatedWalletStatsQuery>,
    ) -> Result<AggregatedWalletStats, ApiError> {
        let cache_key = format!("wallet_stats:{}", wallet_address);
        if let Some(cached_response) = self
            .redis_service
            .get_cached::<AggregatedWalletStats>(&cache_key)
            .await?
        {
            return Ok(cached_response);
        }

        let mut url = format!("{}/{}/pnl/total-stats", self.base_url, wallet_address);

        let mut query_params = vec![];
        if let Some(query) = query {
            if let Some(timeframe) = query.timeframe {
                query_params.push(format!("timeframe={}", timeframe));
            }
            if let Some(cex_transfer) = query.cex_transfer {
                query_params.push(format!("cex_transfer={}", cex_transfer));
            }
            query_params.push(format!("chain={}", query.chain.to_string()));

            if !query_params.is_empty() {
                url.push_str(&format!("?{}", query_params.join("&")));
            }
        }
        let response = self
            .client()
            .get(&url)
            .send()
            .await?
            .json::<AggregatedWalletStatsResponse>()
            .await?;

        // Cache the response in Redis for 1 day (86400 seconds)
        self.redis_service
            .set_cached(&cache_key, &response.data, 86400)
            .await?;

        Ok(response.data)
    }
}
