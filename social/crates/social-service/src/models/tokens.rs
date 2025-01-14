use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    apis::api_models::response::TokenValueDataResponse,
    external_services::rust_monorepo::get_latest_w_metadata::LatestTokenMetadataResponse,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[repr(u8)]
pub enum Chain {
    Ethereum,
    #[default]
    Solana,
}

impl ToString for Chain {
    fn to_string(&self) -> String {
        match self {
            Chain::Ethereum => "ethereum".to_string(),
            Chain::Solana => "solana".to_string(),
        }
    }
}

impl From<String> for Chain {
    fn from(chain_type: String) -> Self {
        match chain_type.to_lowercase().as_str() {
            "ethereum" => Chain::Ethereum,
            _ => Chain::Solana,
        }
    }
}

#[derive(Clone, Debug, PartialEq, FromRow, Serialize, Deserialize, Default, ToSchema)]
pub struct Token {
    /// The token address
    pub address: String,
    /// The token name
    pub name: String,
    /// The token symbol
    pub symbol: String,
    /// The token chain
    pub chain: String,
    /// The token market cap
    pub market_cap: Option<Decimal>,
    /// The token volume 24h USD
    pub volume_24h: Option<Decimal>,
    /// The token liquidity
    pub liquidity: Option<Decimal>,
    /// The logo URI
    pub logo_uri: Option<String>,
}

impl Token {
    pub fn new(
        address: String,
        name: String,
        symbol: String,
        chain: String,
        market_cap: Option<Decimal>,
        volume_24h: Option<Decimal>,
        liquidity: Option<Decimal>,
        logo_uri: Option<String>,
    ) -> Self {
        Self {
            address,
            name,
            symbol,
            chain,
            market_cap,
            volume_24h,
            liquidity,
            logo_uri,
        }
    }
}

impl From<LatestTokenMetadataResponse> for Token {
    fn from(token_info: LatestTokenMetadataResponse) -> Self {
        Self::new(
            token_info.address,
            token_info.token_info.name,
            token_info.token_info.symbol,
            Chain::Solana.to_string(),
            Some(token_info.market_cap),
            token_info.metadata.v_24h_usd,
            token_info.metadata.liquidity,
            token_info.token_info.image_url,
        )
    }
}

impl From<TokenValueDataResponse> for Token {
    fn from(token_value_data: TokenValueDataResponse) -> Self {
        Self {
            address: token_value_data.address,
            chain: token_value_data.chain,
            volume_24h: Some(token_value_data.volume),
            liquidity: Some(token_value_data.liquidity),
            market_cap: Some(token_value_data.market_cap),
            ..Default::default()
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TokenPickRequest {
    pub telegram_message_id: String,
    pub telegram_user_id: String,
    pub telegram_chat_id: String,
    pub telegram_username: Option<String>,
    pub user_bullpen_id: Option<Uuid>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub timestamp: DateTime<Utc>,
    pub address: String,
}
