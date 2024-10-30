use std::{collections::HashMap, sync::Arc};

use chrono::Utc;
use rust_decimal::Decimal;
use tracing::{error, info};

use crate::{
    external_services::{birdeye::BirdeyeService, rust_monorepo::RustMonorepoService},
    models::{
        profiles::{ProfileDetailsResponse, ProfilePickSummary},
        token_picks::{ProfilePicksAndStatsQuery, TokenPick},
        user_stats::{BestPick, UserStats},
    },
    repositories::{token_repository::TokenRepository, user_repository::UserRepository},
    utils::api_errors::ApiError,
};

#[derive(Clone)]
pub struct ProfileService {
    user_repository: Arc<UserRepository>,
    token_repository: Arc<TokenRepository>,
    rust_monorepo_service: Arc<RustMonorepoService>,
    birdeye_service: Arc<BirdeyeService>,
}

impl ProfileService {
    pub fn new(
        user_repository: Arc<UserRepository>,
        token_repository: Arc<TokenRepository>,
        rust_monorepo_service: Arc<RustMonorepoService>,
        birdeye_service: Arc<BirdeyeService>,
    ) -> Self {
        ProfileService {
            user_repository,
            token_repository,
            rust_monorepo_service,
            birdeye_service,
        }
    }

    pub async fn get_profile(&self, username: &str) -> Result<ProfileDetailsResponse, ApiError> {
        let (_, stats) = self
            .get_user_picks_and_stats(username, &ProfilePicksAndStatsQuery::default())
            .await?;
        let response = ProfileDetailsResponse {
            username: username.to_string(),
            name: username.to_string(),
            avatar_url: String::new(),
            pick_summary: ProfilePickSummary::from(stats),
            ..Default::default()
        };
        Ok(response)
    }

    pub async fn get_user_picks_and_stats(
        &self,
        username: &str,
        params: &ProfilePicksAndStatsQuery,
    ) -> Result<(Vec<TokenPick>, UserStats), ApiError> {
        // Get user and their picks
        let user = self
            .user_repository
            .find_by_username(&username)
            .await?
            .ok_or(ApiError::UserNotFound)?;

        let mut picks = self
            .token_repository
            .list_token_picks_by_user_id(user.id, params)
            .await?;

        if picks.is_empty() {
            return Ok((vec![], UserStats::default()));
        }

        let mut picks_to_update = HashMap::with_capacity(picks.len());

        let latest_prices = self
            .rust_monorepo_service
            .get_latest_w_metadata(picks.iter().map(|p| p.token.address.clone()).collect())
            .await?;

        for pick in &mut picks {
            let latest_price = latest_prices
                .get(&pick.token.address)
                .ok_or_else(|| ApiError::InternalServerError("Price data not found".to_string()))?;

            let highest_price = self
                .birdeye_service
                .get_ohlcv_request(
                    &pick.token.chain,
                    &pick.token.address,
                    pick.call_date.timestamp(),
                    latest_price.price_fetched_at_unix_time,
                    "15m",
                )
                .await?;

            let highest_market_cap =
                highest_price.high * latest_price.metadata.supply.unwrap_or_default();

            // Update pick if we found a new highest market cap
            if highest_market_cap > pick.market_cap_at_call {
                pick.highest_market_cap = Some(highest_market_cap);
                picks_to_update.insert(pick.id, pick.clone());
            }

            // Check and update 2x hit status
            let hit_2x = calculate_return(pick, highest_market_cap) >= Decimal::from(2);
            if hit_2x && pick.hit_date.is_none() {
                pick.hit_date = Some(Utc::now().into());
                picks_to_update.insert(pick.id, pick.clone());
            }
        }

        // Calculate stats
        let total_picks = picks.len() as i32;
        let total_hits = picks.iter().filter(|p| p.hit_date.is_some()).count() as i32;

        let hit_rate = if total_picks > 0 {
            let hits_2x = picks
                .iter()
                .filter(|p| {
                    calculate_return(p, p.highest_market_cap.unwrap_or_default())
                        >= Decimal::from(2)
                })
                .count() as i32;
            Decimal::from(hits_2x * 100) / Decimal::from(total_picks)
        } else {
            Decimal::ZERO
        };

        // Calculate returns and find best pick
        let (pick_returns, best_pick) = picks.iter().fold(
            (Decimal::ZERO, None::<BestPick>),
            |(acc_returns, best), pick| {
                let highest_market_cap = pick.highest_market_cap.unwrap_or_default();
                let current_return = calculate_return(pick, highest_market_cap);
                let best_pick = BestPick {
                    token_symbol: pick.token.symbol.clone(),
                    token_address: pick.token.address.clone(),
                    multiplier: current_return,
                };
                let new_best = match best {
                    Some(b) if current_return > b.multiplier => Some(best_pick),
                    None => Some(best_pick),
                    _ => best,
                };
                (acc_returns + current_return, new_best)
            },
        );

        let average_pick_return = if total_picks > 0 {
            pick_returns / Decimal::from(total_picks)
        } else {
            Decimal::ZERO
        };

        let stats = UserStats {
            total_picks,
            hit_rate,
            pick_returns,
            average_pick_return,
            realized_profit: Decimal::ZERO,     // TODO: Implement
            total_volume_traded: Decimal::ZERO, // TODO: Implement
            hits: total_hits,
            misses: total_picks - total_hits,
            best_pick: best_pick.unwrap_or_default(),
        };

        // Update picks in database if needed
        if !picks_to_update.is_empty() {
            info!("Updating {} token picks", picks_to_update.len());
            if let Err(e) = self
                .token_repository
                .update_token_picks(picks_to_update.values().cloned().collect())
                .await
            {
                error!("Failed to update token picks: {}", e);
            }
        }

        Ok((picks, stats))
    }
}

fn calculate_return(pick: &TokenPick, highest_market_cap: Decimal) -> Decimal {
    if highest_market_cap > pick.market_cap_at_call {
        highest_market_cap / pick.market_cap_at_call
    } else {
        Decimal::ZERO
    }
}
