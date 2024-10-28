use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[repr(u8)]
pub enum Chain {
    Ethereum,
    #[default]
    Solana,
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
}

impl Token {
    pub fn new(address: String, name: String, symbol: String, chain: String) -> Self {
        Self {
            address,
            name,
            symbol,
            chain,
        }
    }
}
