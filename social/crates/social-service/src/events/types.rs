use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

use crate::models::token_picks::TokenPick;

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
    type Error = crate::utils::api_errors::ApiError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "social.token_pick" => Ok(Channel::TokenPick),
            _ => Err(crate::utils::api_errors::ApiError::InternalError(format!(
                "Unknown channel: {}",
                value
            ))),
        }
    }
}

impl Display for Channel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Channel::TokenPick => write!(f, "social.token_pick"),
        }
    }
}
