use config::{Config, Environment};
use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct Settings {
    pub environment: Option<String>,
    pub database_url: String,
    pub redis_url: String,
    pub port: Option<u16>,
    pub rust_monorepo_url: String,
    pub birdeye_api_key: String,
    pub cielo_api_key: String,
    pub telegram_bot_token: String,
    pub pg_listen_channels: Option<String>,
    pub ext_data_services_v1_api_key: Option<String>,
    pub ext_data_services_v1_base_url: Option<String>,
    pub usergate_url: String,
    pub usergate_api_key: Option<String>,
    pub s3_bucket: String,
    pub aws_region: Option<String>,
    pub aws_access_key_id: String,
    pub aws_secret_access_key: String,
}

pub fn load_settings() -> Result<Settings, config::ConfigError> {
    let builder = Config::builder()
        .set_default("s3_bucket", "bullpen-social-service")?
        .set_default("aws_region", "us-east-1")?;
    let settings = builder.add_source(Environment::default());
    settings.build()?.try_deserialize()
}
