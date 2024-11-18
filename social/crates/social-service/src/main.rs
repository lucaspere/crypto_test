use std::sync::Arc;

use dotenv::dotenv;
use social_service::{jobs, settings, start_event_listeners};
use tokio::net::TcpListener;
use tracing::{debug, error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let settings = settings::load_settings().expect("Failed to load settings");
    social_service::init_tracing(&settings);
    let port = settings.port.unwrap_or(3000);

    if settings.environment == Some("DEV".to_string()) {
        debug!("Running in DEV environment");
    }
    let (app, container) = social_service::setup_router(&settings).await?;
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    debug!("Server running on http://{:?}", listener.local_addr());
    let settings = Arc::new(settings);

    if settings.environment != Some("DEV".to_string()) {
        jobs::start_background_jobs(container.clone()).await;
        tokio::spawn(async move {
            start_event_listeners(settings, container)
                .await
                .map_err(|e| error!("Failed to start event listeners: {}", e))
        });
    }

    if let Err(e) = axum::serve(listener, app).await {
        error!("Server error: {}", e);
    }

    Ok(())
}
