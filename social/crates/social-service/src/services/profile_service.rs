use std::sync::Arc;

use chrono::Utc;
use rust_decimal::{prelude::One, Decimal};
use tracing::info;

use crate::{
    apis::token_handlers::TokenQuery,
    external_services::{birdeye::BirdeyeService, rust_monorepo::RustMonorepoService},
    models::{
        profiles::{ProfileDetailsResponse, ProfilePickSummary},
        token_picks::{ProfilePicksAndStatsQuery, TokenPickResponse},
        user_stats::{BestPick, UserStats},
    },
    repositories::{token_repository::TokenRepository, user_repository::UserRepository},
    utils::api_errors::ApiError,
};

use super::{redis_service::RedisService, token_service::TokenService};

const CACHE_TTL_SECONDS: u64 = 300; // 5

#[derive(Clone)]
pub struct ProfileService {
    user_repository: Arc<UserRepository>,
    token_repository: Arc<TokenRepository>,
    rust_monorepo_service: Arc<RustMonorepoService>,
    birdeye_service: Arc<BirdeyeService>,
    redis_service: Arc<RedisService>,
    token_service: Arc<TokenService>,
}

impl ProfileService {
    pub fn new(
        user_repository: Arc<UserRepository>,
        token_repository: Arc<TokenRepository>,
        rust_monorepo_service: Arc<RustMonorepoService>,
        birdeye_service: Arc<BirdeyeService>,
        redis_service: Arc<RedisService>,
        token_service: Arc<TokenService>,
    ) -> Self {
        ProfileService {
            user_repository,
            token_repository,
            rust_monorepo_service,
            birdeye_service,
            redis_service,
            token_service,
        }
    }

    pub async fn get_profile(&self, username: &str) -> Result<ProfileDetailsResponse, ApiError> {
        let user = self
            .user_repository
            .find_by_username(username)
            .await?
            .ok_or(ApiError::UserNotFound)?;
        let (_, stats) = self
            .get_user_picks_and_stats(&ProfilePicksAndStatsQuery {
                username: username.to_string(),
                ..Default::default()
            })
            .await?;
        let response = ProfileDetailsResponse {
            id: user.id,
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
        params: &ProfilePicksAndStatsQuery,
    ) -> Result<(Vec<TokenPickResponse>, UserStats), ApiError> {
        info!("Getting user picks and stats for {}", params.username);

        let paramsx = TokenQuery {
            username: Some(params.username.clone()),
            get_all: Some(true),
            ..Default::default()
        };

        let (mut picks, _) = self.token_service.list_token_picks(paramsx).await?;

        if picks.is_empty() {
            info!("No picks found for user {}", params.username);
            return Ok((vec![], UserStats::default()));
        }

        info!("Found {} picks for user {}", picks.len(), params.username);

        let mut total_returns = Decimal::ZERO;
        let mut best_pick = None::<BestPick>;
        let mut hits_2x = 0;

        for pick in &mut picks {
            // Check and update 2x hit status
            let hit_2x = calculate_return(
                &pick.market_cap_at_call,
                &pick.highest_mc_post_call.unwrap_or_default(),
            ) >= Decimal::from(2);
            if hit_2x {
                hits_2x += 1;
                if pick.hit_date.is_none() {
                    info!("Token {} hit 2x", pick.token.symbol);
                    pick.hit_date = Some(Utc::now().into());
                }
            }

            // Update best pick and total returns
            let current_return = calculate_return(
                &pick.market_cap_at_call,
                &pick.highest_mc_post_call.unwrap_or_default(),
            );
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
        }

        let total_picks = picks.len() as i32;
        let total_hits = picks.iter().filter(|p| p.hit_date.is_some()).count() as i32;

        let hit_rate = if total_picks > 0 && hits_2x > 0 {
            Decimal::from(hits_2x * 100) / Decimal::from(total_picks)
        } else {
            Decimal::ZERO
        };

        let average_pick_return = if total_picks > 0 && !total_returns.is_zero() {
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
            params.username,
            total_picks,
            hit_rate,
            total_hits,
            total_picks - total_hits
        );

        let result = (picks, stats);

        Ok(result)
    }
}

fn calculate_return(market_cap_at_call: &Decimal, highest_market_cap: &Decimal) -> Decimal {
    if market_cap_at_call.is_zero() || highest_market_cap.is_zero() {
        Decimal::one()
    } else {
        highest_market_cap / market_cap_at_call
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal::prelude::FromPrimitive;

    use super::*;

    #[test]
    fn test_calculate_return() {
        let market_cap_at_call = Decimal::from_f64(11198827442.235176380912373735).unwrap();
        let _ = Decimal::from_f64(200.5353).unwrap();
        let rounded = market_cap_at_call.round_dp(8);
        assert_eq!(rounded, Decimal::from_f64(11198827442.24).unwrap());
    }
}
