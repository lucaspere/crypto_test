use config::{Config, Environment};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
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
}

pub fn load_settings() -> Result<Settings, config::ConfigError> {
    let settings = Config::builder();
    // let settings = settings.add_source(File::with_name("config"));
    let settings = settings.add_source(Environment::default());
    settings.build()?.try_deserialize()
}
