use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

use crate::{models::token_picks::TokenPick, utils::errors::app_error::AppError};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenPickEventData {
    pub event_date: DateTime<Utc>,
    pub group_name: String,
    pub token_pick: TokenPick,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", untagged)]
pub enum EventData {
    TokenPick(TokenPickEventData),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventMessage {
    pub event_name: String,
    pub data: EventData,
}

#[derive(PartialEq, Eq, Debug, Hash)]
pub enum Channel {
    TokenPick,
}

impl TryFrom<&str> for Channel {
    type Error = AppError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "social.token_picks" => Ok(Channel::TokenPick),
            _ => Err(AppError::InternalServerError()),
        }
    }
}

impl Display for Channel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Channel::TokenPick => write!(f, "social.token_picks"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenPriceMetadata {
    pub price: Option<String>,
    pub symbol: String,
    pub address: String,
    pub metadata: TokenMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenMetadata {
    pub mc: Option<String>,
    pub v24h_usd: Option<String>,
    pub price_change_1h_percent: Option<String>,
    pub price_change_4h_percent: Option<String>,
    pub price_change_24h_percent: Option<String>,
    pub holder: Option<String>,
    pub liquidity: Option<String>,
}

pub struct MessageResult {
    pub message_text: String,
    pub common_fields: String,
}
