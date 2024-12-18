pub mod multi_price;
pub mod ohlcv;

use reqwest::Client;

pub struct BirdeyeService {
    client: Client,
    api_key: String,
}

impl BirdeyeService {
    pub fn new(api_key: String) -> Self {
        let client = Client::new();
        Self { client, api_key }
    }
}
