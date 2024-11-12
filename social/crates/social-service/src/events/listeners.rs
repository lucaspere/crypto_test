use sqlx::{postgres::PgListener, PgPool};
use std::{collections::HashMap, sync::Arc};
use tracing::{debug, error, info, warn};

use crate::{settings::Settings, utils::api_errors::ApiError};

use super::{handlers::EventHandler, types::Channel};

pub struct PostgresEventListener {
    listener: PgListener,
    handlers: HashMap<Channel, Box<dyn EventHandler>>,
}

impl PostgresEventListener {
    pub async fn new(
        settings: Arc<Settings>,
        handlers: HashMap<Channel, Box<dyn EventHandler>>,
    ) -> Result<Self, ApiError> {
        info!("Initializing PostgresEventListener");
        let pool = PgPool::connect(&settings.database_url).await?;
        let listener = PgListener::connect_with(&pool).await?;

        Ok(Self { listener, handlers })
    }

    pub async fn start(&mut self) -> Result<(), ApiError> {
        let channels: Vec<String> = self.handlers.keys().map(|c| c.to_string()).collect();

        if channels.is_empty() {
            warn!("No channels specified for listening");
            return Ok(());
        }

        info!("Listening on channels: {:?}", channels);
        self.listener
            .listen_all(channels.iter().map(|c| c.as_str()).collect::<Vec<&str>>())
            .await?;

        self.process_notifications().await;
        Ok(())
    }

    async fn process_notifications(&mut self) {
        info!("Starting to process notifications");
        while let Ok(notification) = self.listener.recv().await {
            debug!(
                "Received notification on channel: {}",
                notification.channel()
            );

            match Channel::try_from(notification.channel()) {
                Ok(channel) => {
                    if let Some(handler) = self.handlers.get(&channel) {
                        if let Err(e) = handler.handle(notification.payload()).await {
                            error!("Error handling notification: {}", e);
                        }
                    }
                }
                Err(e) => error!("Failed to process notification: {}", e),
            }
        }
    }
}
