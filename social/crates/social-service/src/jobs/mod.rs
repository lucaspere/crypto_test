pub mod token_picks;

use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::error;

use crate::container::ServiceContainer;

pub async fn start_background_jobs(app_state: Arc<ServiceContainer>) {
    let app_state = app_state.clone();
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(600)); // 10 minutes
        interval.tick().await; // Add immediate first tick

        loop {
            if let Err(e) = token_picks::process_token_picks_job(&app_state).await {
                error!("Error processing token picks: {}", e);
            }

            interval.tick().await; // Move tick to end of loop
        }
    });
}
