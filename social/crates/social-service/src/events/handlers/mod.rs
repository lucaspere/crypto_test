pub mod token_pick;

use async_trait::async_trait;
use chrono::{DateTime, FixedOffset, Utc};
use token_pick::TokenPickHandler;
use tracing::{debug, error, instrument};

use crate::utils::api_errors::ApiError;

use super::types::TokenPickEventData;

#[async_trait]
pub trait EventHandler: Send + Sync {
    async fn handle(&self, payload: &str) -> Result<(), ApiError>;
}

#[async_trait]
impl EventHandler for TokenPickHandler {
    #[instrument(skip(self, payload))]
    async fn handle(&self, payload: &str) -> Result<(), ApiError> {
        match serde_json::from_str::<TokenPickEventData>(payload) {
            Ok(data) => {
                debug!(
                    "Processing token:pick event for token {}",
                    data.token_pick.token.symbol
                );
                self.notify_followers(&data).await?;
                Ok(())
            }
            Err(e) => {
                error!("Failed to parse token pick payload: {}", e);
                Err(ApiError::InternalError(e.to_string()))
            }
        }
    }
}

pub fn format_number_with_metric_prefix(num: f64) -> String {
    if num >= 1_000_000_000.0 {
        format!("{:.1}B", num / 1_000_000_000.0)
    } else if num >= 1_000_000.0 {
        format!("{:.1}M", num / 1_000_000.0)
    } else if num >= 1_000.0 {
        format!("{:.1}K", num / 1_000.0)
    } else {
        format!("{:.1}", num)
    }
}

pub fn format_number_with_dynamic_precision(
    num: f64,
    min_precision: u8,
    max_precision: u8,
) -> String {
    let mut precision = min_precision;
    while precision <= max_precision {
        let formatted = format!("{:.1$}", num, precision as usize);
        if formatted.parse::<f64>().unwrap_or(0.0) != 0.0 {
            return formatted;
        }
        precision += 1;
    }
    format!("{:.1$}~", num, max_precision as usize)
}

pub fn format_time_elapsed(call_time: DateTime<FixedOffset>) -> String {
    let now = Utc::now();
    let call_datetime = DateTime::from_timestamp(call_time.timestamp(), 0).unwrap_or_else(|| now);

    let duration = now.signed_duration_since(call_datetime);

    if duration.num_minutes() < 1 {
        "just now".to_string()
    } else if duration.num_hours() < 1 {
        format!("{} minutes ago", duration.num_minutes())
    } else if duration.num_days() < 1 {
        format!("{} hours ago", duration.num_hours())
    } else {
        format!("{} days ago", duration.num_days())
    }
}

pub fn format_header_line(text: &str, is_new_tip: bool) -> String {
    let padding = if is_new_tip { "=" } else { "-" };
    let target_length = 32;

    if text.len() >= target_length {
        return if is_new_tip {
            format!("<b>{}</b>", text)
        } else {
            text.to_string()
        };
    }

    let remaining_space = target_length - text.len() - 2; // -2 for spaces
    let padding_each_side = remaining_space / 2;

    let padded_text = format!(
        "{} {} {}",
        padding.repeat(padding_each_side),
        text,
        padding.repeat(padding_each_side)
    );

    if is_new_tip {
        format!("<b>{}</b>", padded_text)
    } else {
        padded_text
    }
}
pub fn format_risk_score_emoji(score: f64) -> &'static str {
    match score {
        s if s >= 80.0 => "🟢",
        s if s >= 60.0 => "🟡",
        s if s >= 40.0 => "🟠",
        _ => "🔴",
    }
}
