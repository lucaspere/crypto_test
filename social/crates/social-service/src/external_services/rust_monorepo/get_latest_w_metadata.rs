use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LatestTokenMetadataResponse {
    pub address: String,
    pub price: Decimal,
    pub price_fetched_at_unix_time: i64,
    pub market_cap: Decimal,
    pub metadata_fetched_at_unix_time: i64,
    pub metadata: BirdEyeMetadataDataProperty,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct BirdEyeMetadataDataProperty {
    pub name: String,
    pub symbol: String,
    pub decimals: usize,
    #[serde(rename = "logoURI")]
    pub logo_uri: Option<String>,
    extensions: Option<BirdEyeMetadataExtensionsProperty>,
    pub liquidity: Option<Decimal>,
    last_trade_unix_time: Option<i64>,
    last_trade_human_time: Option<String>,
    price_change_1h_percent: Option<Decimal>,
    price_change_4h_percent: Option<Decimal>,
    price_change_12h_percent: Option<Decimal>,
    price_change_24h_percent: Option<Decimal>,
    unique_wallet_1h: Option<Decimal>,
    unique_wallet_1h_change_percent: Option<Decimal>,
    unique_wallet_4h: Option<Decimal>,
    unique_wallet_4h_change_percent: Option<Decimal>,
    unique_wallet_12h: Option<Decimal>,
    unique_wallet_12h_change_percent: Option<Decimal>,
    unique_wallet_24h: Option<Decimal>,
    unique_wallet_24h_change_percent: Option<Decimal>,
    pub supply: Option<Decimal>,
    pub mc: Option<Decimal>,
    circulating_supply: Option<Decimal>,
    real_mc: Option<Decimal>,
    holder: Option<Decimal>,
    pub trade_1h: Option<Decimal>,
    sell_1h: Option<Decimal>,
    buy_1h: Option<Decimal>,
    trade_24h: Option<Decimal>,
    sell_24h: Option<Decimal>,
    buy_24h: Option<Decimal>,
    #[serde(rename = "v24hUSD")]
    pub v_24h_usd: Option<Decimal>,
    #[serde(rename = "vBuy24hUSD")]
    pub v_buy_24h_usd: Option<Decimal>,
    #[serde(rename = "vSell24hUSD")]
    pub v_sell_24h_usd: Option<Decimal>,
    // This is coming from a different endpoint - Token - Creation Token Info
    pub block_human_time: Option<String>,
    // This is coming from a different endpoint - Token - Security
    pub top_10_holder_percent: Option<Decimal>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct BirdEyeMetadataExtensionsProperty {
    coingecko_id: Option<String>,
    telegram: Option<String>,
    twitter: Option<String>,
    discord: Option<String>,
    medium: Option<String>,
    website: Option<String>,
    description: Option<String>,
}
