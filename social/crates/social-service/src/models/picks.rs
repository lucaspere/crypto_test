use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct UserPicksResponse {
    data: Vec<Pick>,
    current_page: u32,
    total_pages: u32,
    per_page: u32,
    total_items: u32,
}

#[derive(Serialize, ToSchema)]
pub struct Pick {
    coin_name: String,
    coin_symbol: String,
    picked_days_ago: u32,
    initial_market_cap: u64,
    peak_market_cap: u64,
    current_market_cap: u64,
    buy_option: bool,
}
