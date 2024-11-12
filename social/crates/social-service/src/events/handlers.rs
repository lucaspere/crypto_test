use async_trait::async_trait;
use futures::future::join_all;
use rust_decimal::{prelude::ToPrimitive, Decimal};
use std::sync::Arc;
use tracing::{debug, error, instrument};

use crate::{container::ServiceContainer, utils::api_errors::ApiError};

use super::types::TokenPickEventData;

#[async_trait]
pub trait EventHandler: Send + Sync {
    async fn handle(&self, payload: &str) -> Result<(), ApiError>;
}

pub struct TokenPickHandler {
    services: Arc<ServiceContainer>,
}

impl TokenPickHandler {
    pub fn new(services: Arc<ServiceContainer>) -> Self {
        Self { services }
    }

    fn format_token_pick_message(&self, data: &TokenPickEventData) -> String {
        let token = &data.token_pick.token;
        let user = &data.token_pick.user;

        let market_cap = format_market_cap(data.token_pick.market_cap_at_call);
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
            "https://t.me/{}/app?startapp=tokenChart_{}",
            bot_username, token.address
        );
        format!(
            "🎯 <b>New Token Pick!</b>\n\n\
            👤 Picked by: <b>{}</b>\n\
            👥 Group: <b>{}</b>\n\
            🪙 Token: <b>{}</b> ({})\n\
            💰 Market Cap: <b>${}</b>\n\
            ⛓️ Chain: <b>{}</b>\n\n\
            🚀 <a href='{}'>View on Bullpen</a>",
            user.username,
            data.group_name,
            token.symbol,
            token.name,
            market_cap,
            token.chain.to_uppercase(),
            bullpen_link,
        )
    }

    #[instrument(skip(self, data), fields(user_id = %data.token_pick.user.id))]
    async fn notify_followers(&self, data: &TokenPickEventData) -> Result<(), ApiError> {
        let followers = self
            .services
            .user_service
            .get_followers(data.token_pick.user.id)
            .await?;

        debug!(
            "Notifying {} followers about token pick {}",
            followers.len(),
            data.token_pick.id
        );

        let message = self.format_token_pick_message(data);

        let notification_futures = followers.into_iter().map(|follower| {
            let message = message.clone();
            let telegram_service = self.services.telegram_service.clone();

            async move {
                debug!("Sending notification to follower {}", follower.telegram_id);

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

fn format_market_cap(market_cap: Decimal) -> String {
    let market_cap_f64 = market_cap.to_f64().unwrap_or_default();

    if market_cap_f64 >= 1_000_000_000.0 {
        format!("{:.2}B", market_cap_f64 / 1_000_000_000.0)
    } else if market_cap_f64 >= 1_000_000.0 {
        format!("{:.2}M", market_cap_f64 / 1_000_000.0)
    } else if market_cap_f64 >= 1_000.0 {
        format!("{:.2}K", market_cap_f64 / 1_000.0)
    } else {
        format!("{:.2}", market_cap_f64)
    }
}
