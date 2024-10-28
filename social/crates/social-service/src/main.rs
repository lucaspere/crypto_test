use dotenv::dotenv;
use social_service::settings;
use tokio::net::TcpListener;
use tracing::debug;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    social_service::init_tracing();

    let settings = settings::load_settings().expect("Failed to load settings");
    let port = settings.port.unwrap_or(3005);

    if settings.environment == Some("DEV".to_string()) {
        debug!("Running in DEV environment");
    }

    let app = social_service::setup_router(&settings).await?;

    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
    debug!("Server running on http://127.0.0.1:{}", port);

    axum::serve(listener, app).await?;

    Ok(())
}
