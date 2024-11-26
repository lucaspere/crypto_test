use reqwest::Client;

pub mod user_trading_stats;

pub struct UserGateService {
    client: Client,
    api_key: Option<String>,
    base_url: String,
}

impl UserGateService {
    pub fn new(base_url: String, api_key: Option<String>) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url,
        }
    }
}
