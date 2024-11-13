use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenReportResponse {
    #[serde(flatten)]
    pub data: HashMap<String, Option<TokenReportData>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenReportData {
    #[serde(rename = "cached_at")]
    pub cached_at: Option<String>,
    pub creator: Option<String>,
    #[serde(rename = "detectedAt")]
    pub detected_at: Option<String>,
    pub events: Option<Vec<TokenEvent>>,
    #[serde(rename = "fileMeta")]
    pub file_meta: Option<TokenFileMeta>,
    #[serde(rename = "freezeAuthority")]
    pub freeze_authority: Option<String>,
    #[serde(rename = "knownAccounts")]
    pub known_accounts: Option<HashMap<String, KnownAccount>>,
    #[serde(rename = "liquidityPools")]
    pub liquidity_pools: Option<Vec<LiquidityPool>>,
    pub mint: String,
    #[serde(rename = "mintAuthority")]
    pub mint_authority: Option<String>,
    pub risks: Option<Vec<TokenRisk>>,
    pub rugged: bool,
    pub score: f64,
    pub token: TokenInfo,
    #[serde(rename = "tokenMeta")]
    pub token_meta: TokenMetadata,
    #[serde(rename = "tokenProgram")]
    pub token_program: String,
    #[serde(rename = "tokenType")]
    pub token_type: String,
    pub token_extensions: Option<String>,
    #[serde(rename = "topHolders")]
    pub top_holders: Vec<TokenHolder>,
    #[serde(rename = "totalLPProviders")]
    pub total_lp_providers: Option<i32>,
    #[serde(rename = "totalMarketLiquidity")]
    pub total_market_liquidity: Option<f64>,
    #[serde(rename = "transferFee")]
    pub transfer_fee: Option<TransferFee>,
    pub verification: Option<TokenVerification>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenEvent {
    pub event: i32,
    pub old_value: String,
    pub new_value: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenFileMeta {
    pub description: String,
    pub name: String,
    pub symbol: String,
    pub image: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnownAccount {
    pub name: String,
    #[serde(rename = "type")]
    pub account_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiquidityPool {
    pub pubkey: String,
    #[serde(rename = "marketType")]
    pub market_type: String,
    #[serde(rename = "mintA")]
    pub mint_a: String,
    #[serde(rename = "mintB")]
    pub mint_b: String,
    #[serde(rename = "mintLP")]
    pub mint_lp: String,
    #[serde(rename = "liquidityA")]
    pub liquidity_a: String,
    #[serde(rename = "liquidityB")]
    pub liquidity_b: String,
    #[serde(rename = "mintAAccount")]
    pub mint_a_account: TokenAccountInfo,
    #[serde(rename = "mintBAccount")]
    pub mint_b_account: TokenAccountInfo,
    #[serde(rename = "mintLPAccount")]
    pub mint_lp_account: TokenAccountInfo,
    #[serde(rename = "liquidityAAccount")]
    pub liquidity_a_account: TokenAccountInfo,
    #[serde(rename = "liquidityBAccount")]
    pub liquidity_b_account: TokenAccountInfo,
    pub lp: LPInfo,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenAccountInfo {
    #[serde(rename = "mintAuthority")]
    pub mint_authority: Option<String>,
    pub supply: i64,
    pub decimals: i32,
    #[serde(rename = "isInitialized")]
    pub is_initialized: bool,
    #[serde(rename = "freezeAuthority")]
    pub freeze_authority: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LPInfo {
    #[serde(rename = "baseMint")]
    pub base_mint: String,
    #[serde(rename = "quoteMint")]
    pub quote_mint: String,
    #[serde(rename = "lpMint")]
    pub lp_mint: String,
    #[serde(rename = "quotePrice")]
    pub quote_price: f64,
    #[serde(rename = "basePrice")]
    pub base_price: f64,
    pub base: f64,
    pub quote: f64,
    #[serde(rename = "reserveSupply")]
    pub reserve_supply: f64,
    #[serde(rename = "currentSupply")]
    pub current_supply: f64,
    #[serde(rename = "quoteUSD")]
    pub quote_usd: f64,
    #[serde(rename = "baseUSD")]
    pub base_usd: f64,
    #[serde(rename = "pctReserve")]
    pub pct_reserve: f64,
    #[serde(rename = "pctSupply")]
    pub pct_supply: f64,
    pub holders: Option<Vec<TokenHolder>>,
    #[serde(rename = "totalTokensUnlocked")]
    pub total_tokens_unlocked: f64,
    #[serde(rename = "tokenSupply")]
    pub token_supply: f64,
    #[serde(rename = "lpLocked")]
    pub lp_locked: f64,
    #[serde(rename = "lpUnlocked")]
    pub lp_unlocked: f64,
    #[serde(rename = "lpLockedPct")]
    pub lp_locked_pct: f64,
    #[serde(rename = "lpLockedUSD")]
    pub lp_locked_usd: f64,
    #[serde(rename = "lpMaxSupply")]
    pub lp_max_supply: f64,
    #[serde(rename = "lpCurrentSupply")]
    pub lp_current_supply: f64,
    #[serde(rename = "lpTotalSupply")]
    pub lp_total_supply: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenRisk {
    pub name: String,
    pub value: String,
    pub description: String,
    pub score: f64,
    pub level: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenInfo {
    #[serde(rename = "mintAuthority")]
    pub mint_authority: Option<String>,
    pub supply: i64,
    pub decimals: i32,
    #[serde(rename = "isInitialized")]
    pub is_initialized: bool,
    #[serde(rename = "freezeAuthority")]
    pub freeze_authority: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenMetadata {
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub mutable: bool,
    #[serde(rename = "updateAuthority")]
    pub update_authority: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenHolder {
    pub address: String,
    pub amount: i64,
    pub decimals: i32,
    pub pct: f64,
    #[serde(rename = "uiAmount")]
    pub ui_amount: f64,
    #[serde(rename = "uiAmountString")]
    pub ui_amount_string: String,
    pub owner: String,
    pub insider: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransferFee {
    pub pct: f64,
    #[serde(rename = "maxAmount")]
    pub max_amount: f64,
    pub authority: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenVerification {
    pub mint: String,
    pub payer: String,
    pub name: String,
    pub symbol: String,
    pub description: String,
    pub jup_verified: bool,
    pub links: Vec<VerificationLink>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerificationLink {
    #[serde(rename = "type")]
    pub link_type: String,
    pub url: String,
}
