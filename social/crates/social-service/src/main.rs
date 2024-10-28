use dotenv::dotenv;
use social_service::settings;
use tokio::net::TcpListener;
use tracing::{debug, error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    social_service::init_tracing();

    let settings = settings::load_settings().expect("Failed to load settings");
    let port = settings.port.unwrap_or(3000);

    if settings.environment == Some("DEV".to_string()) {
        debug!("Running in DEV environment");
    }

    let app = social_service::setup_router(&settings).await?;

    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    debug!(
        "Server running on http://{}",
        listener.local_addr().unwrap()
    );

    if let Err(e) = axum::serve(listener, app).await {
        error!("Server error: {}", e);
    }

    Ok(())
}
