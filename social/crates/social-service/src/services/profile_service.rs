use std::{collections::HashMap, sync::Arc};

use chrono::Utc;
use futures::future::join_all;
use rust_decimal::Decimal;
use tracing::{debug, error, info};

use crate::{
    external_services::{birdeye::BirdeyeService, rust_monorepo::RustMonorepoService},
    models::{
        profiles::{ProfileDetailsResponse, ProfilePickSummary},
        token_picks::{ProfilePicksAndStatsQuery, TokenPickResponse},
        user_stats::{BestPick, UserStats},
    },
    repositories::{token_repository::TokenRepository, user_repository::UserRepository},
    utils::api_errors::ApiError,
};

use super::redis_service::RedisService;

const CACHE_TTL_SECONDS: u64 = 300; // 5

#[derive(Clone)]
pub struct ProfileService {
    user_repository: Arc<UserRepository>,
    token_repository: Arc<TokenRepository>,
    rust_monorepo_service: Arc<RustMonorepoService>,
    birdeye_service: Arc<BirdeyeService>,
    redis_service: Arc<RedisService>,
}

impl ProfileService {
    pub fn new(
        user_repository: Arc<UserRepository>,
        token_repository: Arc<TokenRepository>,
        rust_monorepo_service: Arc<RustMonorepoService>,
        birdeye_service: Arc<BirdeyeService>,
        redis_service: Arc<RedisService>,
    ) -> Self {
        ProfileService {
            user_repository,
            token_repository,
            rust_monorepo_service,
            birdeye_service,
            redis_service,
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
    ) -> Result<(Vec<TokenPickResponse>, UserStats), ApiError> {
        let cache_key = format!("user_picks_stats:{}", username);
        debug!("Checking cache for key: {}", cache_key);
        if let Ok(Some(cached)) = self
            .redis_service
            .get_cached::<(Vec<TokenPickResponse>, UserStats)>(&cache_key)
            .await
        {
            info!("Cache hit for user picks and stats for {}", username);
            return Ok(cached);
        }
        debug!("Cache miss for key: {}", cache_key);
        info!("Getting user picks and stats for {}", username);

        let user = self
            .user_repository
            .find_by_username(&username)
            .await?
            .ok_or(ApiError::UserNotFound)?;

        let mut picks = self
            .token_repository
            .list_token_picks_by_user_id(user.id, Some(params))
            .await?;

        if picks.is_empty() {
            info!("No picks found for user {}", username);
            return Ok((vec![], UserStats::default()));
        }

        info!("Found {} picks for user {}", picks.len(), username);

        let token_addresses: Vec<_> = picks.iter().map(|p| p.token.address.clone()).collect();
        let latest_prices_future = self
            .rust_monorepo_service
            .get_latest_w_metadata(token_addresses);

        let ohlcv_futures = picks.iter().map(|pick| {
            let birdeye_service = self.birdeye_service.clone();
            async move {
                birdeye_service
                    .get_ohlcv_request(
                        &pick.token.chain,
                        &pick.token.address,
                        pick.call_date.timestamp(),
                        Utc::now().timestamp(), // We'll update this with actual price timestamp
                        "15m",
                    )
                    .await
                    .map(|result| (pick.token.address.clone(), result))
            }
        });

        // Execute price and OHLCV requests concurrently
        let (latest_prices, ohlcv_results) =
            tokio::join!(latest_prices_future, join_all(ohlcv_futures));
        let latest_prices = latest_prices?;
        let ohlcv_map: HashMap<_, _> = ohlcv_results.into_iter().filter_map(Result::ok).collect();

        let mut picks_to_update = HashMap::with_capacity(picks.len());
        let mut pick_responses = Vec::with_capacity(picks.len());
        let mut total_returns = Decimal::ZERO;
        let mut best_pick = None::<BestPick>;
        let mut hits_2x = 0;

        for pick in &mut picks {
            let latest_price = latest_prices
                .get(&pick.token.address)
                .ok_or_else(|| ApiError::InternalServerError("Price data not found".to_string()))?;

            let highest_price = ohlcv_map
                .get(&pick.token.address)
                .ok_or_else(|| ApiError::InternalServerError("OHLCV data not found".to_string()))?;

            let supply = latest_price.metadata.supply.unwrap_or_default();
            let highest_market_cap = highest_price.high * supply;

            if highest_market_cap > pick.highest_market_cap.unwrap_or_default() {
                info!(
                    "New highest market cap for token {}: {}",
                    pick.token.symbol, highest_market_cap
                );
                pick.highest_market_cap = Some(highest_market_cap);
                picks_to_update.insert(pick.id, pick.clone());
            }

            // Check and update 2x hit status
            let hit_2x = calculate_return(
                &pick.market_cap_at_call,
                &pick.highest_market_cap.unwrap_or_default(),
            ) >= Decimal::from(2);
            if hit_2x {
                hits_2x += 1;
                if pick.hit_date.is_none() {
                    info!("Token {} hit 2x", pick.token.symbol);
                    pick.hit_date = Some(Utc::now().into());
                    picks_to_update.insert(pick.id, pick.clone());
                }
            }

            // Create pick response and update stats
            let mut pick_response = TokenPickResponse::from(pick.clone());
            pick_response.current_market_cap =
                latest_price.metadata.mc.unwrap_or_default().round_dp(2);
            pick_response.current_multiplier = calculate_return(
                &pick_response.market_cap_at_call,
                &pick_response.current_market_cap,
            )
            .round_dp(2)
            .to_string()
            .parse::<f32>()
            .unwrap_or_default();

            // Update best pick and total returns
            let current_return = calculate_return(&pick.market_cap_at_call, &highest_market_cap);
            total_returns += current_return;

            let new_best = BestPick {
                token_symbol: pick.token.symbol.clone(),
                token_address: pick.token.address.clone(),
                multiplier: current_return,
            };

            best_pick = match best_pick {
                Some(b) if current_return > b.multiplier => Some(new_best),
                None => Some(new_best),
                b => b,
            };

            pick_responses.push(pick_response);
        }

        let total_picks = pick_responses.len() as i32;
        let total_hits = pick_responses
            .iter()
            .filter(|p| p.hit_date.is_some())
            .count() as i32;

        let hit_rate = if total_picks > 0 {
            Decimal::from(hits_2x * 100) / Decimal::from(total_picks)
        } else {
            Decimal::ZERO
        };

        let average_pick_return = if total_picks > 0 {
            total_returns / Decimal::from(total_picks)
        } else {
            Decimal::ZERO
        };

        let stats = UserStats {
            total_picks,
            hit_rate: hit_rate.round_dp(2),
            pick_returns: total_returns.round_dp(2),
            average_pick_return: average_pick_return.round_dp(2),
            realized_profit: Decimal::ZERO,     // TODO: Implement
            total_volume_traded: Decimal::ZERO, // TODO: Implement
            hits: total_hits,
            misses: total_picks - total_hits,
            best_pick: best_pick.unwrap_or_default(),
        };

        info!(
            "Stats for {}: {} picks, {}% hit rate, {} hits, {} misses",
            username,
            total_picks,
            hit_rate,
            total_hits,
            total_picks - total_hits
        );

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

        let result = (pick_responses, stats);

        // Cache the result
        if let Err(e) = self
            .redis_service
            .set_cached(&cache_key, &result, CACHE_TTL_SECONDS)
            .await
        {
            error!("Failed to cache user picks and stats: {}", e);
        }

        Ok(result)
    }
}

fn calculate_return(market_cap_at_call: &Decimal, highest_market_cap: &Decimal) -> Decimal {
    highest_market_cap / market_cap_at_call
}

#[cfg(test)]
mod tests {
    use rust_decimal::prelude::FromPrimitive;

    use super::*;

    #[test]
    fn test_calculate_return() {
        let market_cap_at_call = Decimal::from_f64(11198827442.235176380912373735).unwrap();
        let highest_market_cap = Decimal::from_f64(200.5353).unwrap();
        let rounded = market_cap_at_call.round_dp(8);
        assert_eq!(rounded, Decimal::from_f64(11198827442.24).unwrap());
    }
}
