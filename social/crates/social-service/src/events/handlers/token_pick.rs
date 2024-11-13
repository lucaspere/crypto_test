use std::sync::Arc;

use futures::future::join_all;
use tracing::{error, info, instrument};

use crate::{
    container::ServiceContainer,
    events::{handlers::format_risk_score_emoji, types::TokenPickEventData},
    external_services::{
        ext_data_services_v1::token_data::types::TokenReportData,
        rust_monorepo::get_latest_w_metadata::LatestTokenMetadataResponse,
    },
    utils::api_errors::ApiError,
};
use rust_decimal::prelude::*;

pub struct TokenPickHandler {
    services: Arc<ServiceContainer>,
}

impl TokenPickHandler {
    pub fn new(services: Arc<ServiceContainer>) -> Self {
        Self { services }
    }

    #[instrument(skip(self, data), fields(user_id = %data.token_pick.user.id))]
    pub(super) async fn notify_followers(&self, data: &TokenPickEventData) -> Result<(), ApiError> {
        let followers = self
            .services
            .user_service
            .get_followers(&data.token_pick.user.username)
            .await?;

        info!(
            "Notifying {} followers about token pick {}",
            followers.len(),
            data.token_pick.id
        );
        let token_price_metadata = self
            .services
            .rust_monorepo_service
            .get_latest_w_metadata(vec![data.token_pick.token.address.clone()])
            .await?
            .into_iter()
            .next()
            .unwrap()
            .1;
        let token_report = if let Some(token_data_service) = &self.services.token_data_service {
            token_data_service
                .get_token_report(&[data.token_pick.token.address.clone()])
                .await
                .ok()
        } else {
            None
        };
        let rugcheck_report_data =
            token_report.and_then(|r| r.data.into_iter().next().and_then(|(_, d)| d));
        let message = self
            .format_token_pick_message(&token_price_metadata.into(), rugcheck_report_data, data)
            .unwrap();

        let notification_futures = followers.into_iter().map(|follower| {
            let message = message.message_text.clone();
            let telegram_service = self.services.telegram_service.clone();

            async move {
                info!("Sending notification to follower {}", follower.telegram_id);

                if let Err(e) = telegram_service
                    .send_message(follower.telegram_id as u64, &message)
                    .await
                {
                    error!(
                        "Failed to send telegram message to {}: {}",
                        follower.telegram_id, e
                    );
                }
            }
        });

        join_all(notification_futures).await;

        Ok(())
    }

    pub fn format_token_pick_message(
        &self,
        token_price_metadata: &TokenPriceMetadata,
        rugcheck_report_data: Option<TokenReportData>,
        event_data: &TokenPickEventData,
    ) -> Result<MessageResult, Box<dyn std::error::Error>> {
        let token_pick = &event_data.token_pick;
        let symbol = &token_price_metadata.symbol;
        let address = &token_price_metadata.address;
        let bot_username = self
            .services
            .telegram_service
            .bot_info
            .as_ref()
            .unwrap()
            .username
            .clone()
            .unwrap_or("BullpenFiBot".to_string());
        let bullpen_link = format!(
            "https://t.me/{}/app?startapp=profile_{}",
            bot_username, token_pick.user.username
        );

        let _original_call_link = match (
            token_pick.telegram_message_id,
            token_pick.group_id.to_string(),
        ) {
            (Some(msg_id), chat) => {
                let formatted_chat_id = if chat.starts_with("-100") {
                    &chat[4..]
                } else if chat.starts_with('-') {
                    &chat[1..]
                } else {
                    &chat
                };
                format!("https://t.me/c/{}/{}", formatted_chat_id, msg_id)
            }
            _ => String::new(),
        };

        let market_cap = token_price_metadata
            .metadata
            .mc
            .as_ref()
            .and_then(|mc| mc.parse::<f64>().ok())
            .map(|mc| format_number_with_metric_prefix(mc))
            .unwrap_or_else(|| "-.-".to_string());

        let volume_24h = token_price_metadata
            .metadata
            .v24h_usd
            .as_ref()
            .and_then(|v| v.parse::<f64>().ok())
            .map(|v| format_number_with_metric_prefix(v))
            .unwrap_or_else(|| "-.-".to_string());

        let price_change_1h = token_price_metadata
            .format_price_change(&token_price_metadata.metadata.price_change_1h_percent);
        let price_change_4h = token_price_metadata
            .format_price_change(&token_price_metadata.metadata.price_change_4h_percent);
        let price_change_24h = token_price_metadata
            .format_price_change(&token_price_metadata.metadata.price_change_24h_percent);

        let holders = token_price_metadata
            .metadata
            .holder
            .as_ref()
            .and_then(|h| h.parse::<f64>().ok())
            .map(|h| format_number_with_metric_prefix(h))
            .unwrap_or_else(|| "-.-".to_string());

        let price = token_price_metadata
            .price
            .as_ref()
            .and_then(|p| p.parse::<f64>().ok())
            .map(|p| format_number_with_dynamic_precision(p, 1, 8))
            .unwrap_or_else(|| "-.-".to_string());

        let liquidity = token_price_metadata
            .metadata
            .liquidity
            .as_ref()
            .and_then(|l| l.parse::<f64>().ok())
            .map(|l| format_number_with_metric_prefix(l))
            .unwrap_or_else(|| "-.-".to_string());

        // Format token symbol with link
        let linked_token_symbol =
            format!(r#"<a href="{}{}">{}</a>"#, bullpen_link, address, symbol);

        // Format risk score
        let risk_score_display = match &rugcheck_report_data {
            Some(report) => {
                if report.score != -1.0 {
                    let formatted_score = report.score.round();
                    let emoji = format_risk_score_emoji(report.score);
                    format!(
                        r#"<a href="https://rugcheck.xyz/tokens/{}">{}</a> {}"#,
                        address, formatted_score, emoji
                    )
                } else {
                    "??? ❌".to_string()
                }
            }
            None => "??? ❌".to_string(),
        };

        // Format top holders
        let (top_holders_display, _total_top_k_percentages) =
            format_top_holders(rugcheck_report_data, Some(&address), 5);

        // Format market cap at call time
        let formatted_market_cap_at_call =
            format_number_with_metric_prefix(token_pick.market_cap_at_call.to_f64().unwrap());

        let header = format_header_line(
            &format!("🎯 {} just made a pick!", token_pick.user.username),
            true,
        );
        // Create common fields and message text
        let common_fields = format!(
            r#"
	Ticker: {}
	{}: <code>{}</code>
	{}: <code>{}</code>
	1h: <code>{}</code> 4h: <code>{}</code> 24h: <code>{}</code>

	Volume (24h): <code>${}</code>
	Liquidity: <code>${}</code>
	Holders: <code>{}</code>
	Top {}: {}
	Rugcheck Score: {}"#,
            linked_token_symbol,
            "Market Cap at Call",
            market_cap,
            "Price at Call",
            price,
            price_change_1h,
            price_change_4h,
            price_change_24h,
            volume_24h,
            liquidity,
            holders,
            5,
            top_holders_display,
            risk_score_display,
        );

        let new_tip_specific_info = format!(
            r#"{} at a <b>${}</b> market cap."#,
            linked_token_symbol, formatted_market_cap_at_call
        );
        let mint_address_copy = format!(r#"<code>${}</code> — <i>tap to copy</i>"#, address);
        let bullpen_link = format!(
            r#"<b><a href="{}">View {} on Bullpen</a></b>"#,
            bullpen_link, event_data.token_pick.user.username
        );

        let message_text = format!(
            "{header}\n\n{new_tip_specific_info}\n{common_fields}\n\n{mint_address_copy}\n\n{bullpen_link}"
        );

        Ok(MessageResult {
            message_text,
            common_fields,
        })
    }
}

fn format_header_line(text: &str, is_new_tip: bool) -> String {
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

use serde::{Deserialize, Serialize};

use super::{format_number_with_dynamic_precision, format_number_with_metric_prefix};

#[derive(Debug, Serialize, Deserialize)]
pub struct CurrencyMeta {
    pub symbol: String,
    pub name: String,
    pub currency: Currency,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Currency {
    pub mint_address: Option<String>,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct TelegramUser {
    pub username: Option<String>,
    pub first_name: Option<String>,
}

pub struct MessageResult {
    pub message_text: String,
    pub common_fields: String,
}

impl From<LatestTokenMetadataResponse> for TokenPriceMetadata {
    fn from(value: LatestTokenMetadataResponse) -> Self {
        let metadata = TokenMetadata {
            mc: Some((value.price * value.metadata.supply.unwrap_or_default()).to_string()),
            v24h_usd: value.metadata.v_24h_usd.map(|v| v.to_string()),
            price_change_1h_percent: value
                .metadata
                .price_change_1h_percent
                .map(|v| v.to_string()),
            price_change_4h_percent: value
                .metadata
                .price_change_4h_percent
                .map(|v| v.to_string()),
            price_change_24h_percent: value
                .metadata
                .price_change_24h_percent
                .map(|v| v.to_string()),
            holder: value.metadata.top_10_holder_percent.map(|v| v.to_string()),
            liquidity: value.metadata.liquidity.map(|v| v.to_string()),
        };
        Self {
            price: Some(value.price.to_string()),
            symbol: value.metadata.symbol,
            address: value.address,
            metadata,
        }
    }
}
impl TokenPriceMetadata {
    fn format_price_change(&self, value: &Option<String>) -> String {
        value
            .as_ref()
            .and_then(|v| v.parse::<f64>().ok())
            .map(|v| format!("{}%", v.round()))
            .unwrap_or_else(|| "-.-".to_string())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenReport {
    pub score: Option<f64>,
    pub top_holders: Vec<TopHolder>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TopHolder {
    pub owner: String,
    pub pct: f64,
}

fn format_top_holders(
    rugcheck_report_data: Option<TokenReportData>,
    mint_address: Option<&str>,
    num_top_holders: usize,
) -> (String, f64) {
    match rugcheck_report_data {
        Some(report) => {
            let mut top_k_percentages: Vec<String> = report
                .top_holders
                .iter()
                .take(num_top_holders)
                .map(|holder| {
                    format!(
                        r#"<a href="https://solscan.io/account/{}">{:.1}</a>"#,
                        holder.owner, holder.pct
                    )
                })
                .collect();

            // Pad with "-" if needed
            while top_k_percentages.len() < num_top_holders {
                top_k_percentages.push("-".to_string());
            }

            let total_percentage: f64 = report
                .top_holders
                .iter()
                .take(num_top_holders)
                .map(|holder| holder.pct)
                .sum();

            let display = format!(
                "{} <b>[{}%]</b>",
                top_k_percentages.join(" | "),
                total_percentage.round()
            );

            (display, total_percentage)
        }
        None => ("No Data Available".to_string(), 0.0),
    }
}
