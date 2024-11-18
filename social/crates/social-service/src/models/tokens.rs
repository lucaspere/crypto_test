use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::external_services::rust_monorepo::get_latest_w_metadata::LatestTokenMetadataResponse;

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
    pub address: String,
    pub name: String,
    pub symbol: String,
    pub chain: String,
    pub volume_24h: Option<Decimal>,
    pub liquidity: Option<Decimal>,
    pub logo_uri: Option<String>,
}

impl Token {
    pub fn new(
        address: String,
        name: String,
        symbol: String,
        chain: String,
        volume_24h: Option<Decimal>,
        liquidity: Option<Decimal>,
        logo_uri: Option<String>,
    ) -> Self {
        Self {
            address,
            name,
            symbol,
            chain,
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
            token_info.metadata.name,
            token_info.metadata.symbol,
            Chain::Solana.to_string(),
            token_info.metadata.v_24h_usd,
            token_info.metadata.liquidity,
            token_info.metadata.logo_uri,
        )
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TokenPickRequest {
    pub telegram_message_id: String,
    pub telegram_user_id: String,
    pub telegram_chat_id: String,
    pub user_bullpen_id: Option<Uuid>,
    pub timestamp: Option<i64>,
    pub address: String,
}
