use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use sqlx::{postgres::PgListener, PgPool};
use std::env;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::models::token_picks::TokenPick;

pub const PG_LISTEN_CHANNELS: &str = "PG_LISTEN_CHANNELS";

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenPickEventData {
    pub user_id: Uuid,
    pub group_id: Option<i64>,
    pub username: String,
    pub telegram_id: i64,
    pub event_date: DateTime<Utc>,
    pub token_pick: TokenPick,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", untagged)]
pub enum EventData {
    UserFollow(FollowEventData),
    TokenPick(TokenPickEventData),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FollowData {
    pub id: Uuid,
    pub username: String,
    pub telegram_id: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FollowEventData {
    pub follower: FollowData,
    pub followed: FollowData,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventMessage {
    pub event_name: String,
    pub data: EventData,
}

impl From<EventData> for EventMessage {
    fn from(event_data: EventData) -> Self {
        let data = event_data;
        match &data {
            EventData::UserFollow(_) => EventMessage {
                event_name: Channel::UserFollow.to_string(),
                data,
            },
            EventData::TokenPick(_) => EventMessage {
                event_name: Channel::TokenPick.to_string(),
                data,
            },
        }
    }
}

pub enum Channel {
    UserFollow,
    TokenPick,
}

impl From<&str> for Channel {
    fn from(value: &str) -> Self {
        match value {
            "user:follow" => Channel::UserFollow,
            "token:pick" => Channel::TokenPick,
            _ => {
                warn!("Unknown channel: {}", value);
                Channel::UserFollow
            }
        }
    }
}

impl ToString for Channel {
    fn to_string(&self) -> String {
        match self {
            Channel::UserFollow => "user:follow".to_string(),
            Channel::TokenPick => "token:pick".to_string(),
        }
    }
}

pub struct PgNotificationListener {
    listener: PgListener,
}

impl PgNotificationListener {
    pub async fn new(url: &str) -> Result<Self, sqlx::Error> {
        info!("Initializing PgNotificationListener");
        let pool = PgPool::connect(url).await?;
        let listener = PgListener::connect_with(&pool).await?;
        Ok(PgNotificationListener { listener })
    }

    pub async fn start_listening(&mut self) -> Result<(), sqlx::Error> {
        let channels = env::var(PG_LISTEN_CHANNELS)
            .map(|s| s.split(',').map(String::from).collect())
            .unwrap_or_else(|_| {
                warn!("{} environment variable is not set", PG_LISTEN_CHANNELS);
                Vec::new()
            });

        if channels.is_empty() {
            warn!("No channels specified for listening");
        } else {
            info!("Listening on channels: {:?}", channels);
            self.listener
                .listen_all(channels.iter().map(|c| c.as_str()).collect::<Vec<&str>>())
                .await
                .map_err(|e| {
                    error!("Failed to listen on channels: {}", e);
                    e
                })?;
        }

        self.process_notifications(PgNotificationHandler::new(Box::new(|payload| {
            debug!("Received payload: {:#?}", payload);
        })))
        .await;

        Ok(())
    }

    pub async fn process_notifications(&mut self, handler: PgNotificationHandler) {
        info!("Starting to process notifications");
        while let Ok(notification) = self.listener.recv().await {
            debug!(
                "Received notification on channel: {}",
                notification.channel()
            );
            let payload = notification.payload();
            match Channel::from(notification.channel()) {
                Channel::UserFollow => handler.handle_user_follow(payload),
                Channel::TokenPick => handler.handle_token_pick(payload),
            }
        }
    }
}

pub struct PgNotificationHandler {
    pub payload_processor: Box<dyn Fn(EventMessage) + Send + Sync>,
}

impl PgNotificationHandler {
    pub fn new(payload_processor: Box<dyn Fn(EventMessage) + Send + Sync>) -> Self {
        PgNotificationHandler { payload_processor }
    }

    fn handle_user_follow(&self, payload: &str) {
        match from_str::<FollowEventData>(payload) {
            Ok(data) => {
                let message = EventMessage {
                    event_name: Channel::UserFollow.to_string(),
                    data: EventData::UserFollow(data),
                };
                debug!("Processing user:follow event");
                (self.payload_processor)(message);
            }
            Err(e) => error!("Failed to parse user follow payload: {}", e),
        }
    }

    fn handle_token_pick(&self, payload: &str) {
        match from_str::<TokenPickEventData>(payload) {
            Ok(data) => {
                let message = EventMessage {
                    event_name: Channel::TokenPick.to_string(),
                    data: EventData::TokenPick(data),
                };
                debug!("Processing token:pick event");
                (self.payload_processor)(message);
            }
            Err(e) => error!("Failed to parse token pick payload: {}", e),
        }
    }
}
