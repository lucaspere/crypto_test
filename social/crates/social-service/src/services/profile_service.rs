use std::{collections::HashSet, sync::Arc};

use chrono::Utc;
use futures::future::join_all;
use rust_decimal::{
    prelude::{One, Zero},
    Decimal,
};
use tracing::info;

use crate::{
    apis::{
        profile_handlers::{LeaderboardQuery, LeaderboardResponse, LeaderboardSort, ProfileQuery},
        token_handlers::TokenQuery,
    },
    external_services::{
        birdeye::BirdeyeService, cielo::CieloService, rust_monorepo::RustMonorepoService,
    },
    models::{
        profiles::{ProfileDetailsResponse, ProfilePickSummary},
        token_picks::{ProfilePicksAndStatsQuery, TokenPickResponse},
        tokens::Chain,
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
    cielo_service: Arc<CieloService>,
}

impl ProfileService {
    pub fn new(
        user_repository: Arc<UserRepository>,
        token_repository: Arc<TokenRepository>,
        rust_monorepo_service: Arc<RustMonorepoService>,
        birdeye_service: Arc<BirdeyeService>,
        redis_service: Arc<RedisService>,
        token_service: Arc<TokenService>,
        cielo_service: Arc<CieloService>,
    ) -> Self {
        ProfileService {
            user_repository,
            token_repository,
            rust_monorepo_service,
            birdeye_service,
            redis_service,
            token_service,
            cielo_service,
        }
    }

    pub async fn get_profile(
        &self,
        params: ProfileQuery,
    ) -> Result<ProfileDetailsResponse, ApiError> {
        info!(
            "Attempting to fetch profile for username: {}",
            params.username
        );
        let cache_key = format!(
            "profile:{}:{}{}",
            params.username,
            params.picked_after.to_string(),
            params
                .group_id
                .map_or(String::new(), |id| format!(":{}", id))
        );
        if let Some(cached_response) = self
            .redis_service
            .get_cached::<ProfileDetailsResponse>(&cache_key)
            .await?
        {
            return Ok(cached_response);
        }

        info!(
            "Cache miss, fetching profile from database for username: {}",
            params.username
        );
        let user = self
            .user_repository
            .find_by_username(&params.username)
            .await?
            .ok_or(ApiError::UserNotFound)?;
        info!(
            "User found, fetching user picks and stats for username: {}",
            params.username
        );
        let (_, mut stats) = self
            .get_user_picks_and_stats(&ProfilePicksAndStatsQuery {
                username: params.username.clone(),
                picked_after: Some(params.picked_after.clone()),
                multiplier: None,
                group_ids: params.group_id.map(|id| vec![id]),
            })
            .await?;

        if let Some(wallet) = user.wallet_addresses.as_ref().and_then(|wa| {
            wa.iter()
                .filter(|w| w.address.is_some())
                .find(|w| w.chain == Some(Chain::Solana.to_string()))
        }) {
            let realized_pnl_usd = self
                .cielo_service
                .get_wallet_stats(wallet.address.as_ref().unwrap(), None)
                .await?;
            stats.realized_profit = realized_pnl_usd.realized_pnl_usd.round_dp(2);
        }
        let response = ProfileDetailsResponse {
            id: user.id,
            username: params.username.clone(),
            name: params.username.clone(),
            avatar_url: String::new(),
            pick_summary: ProfilePickSummary::from(stats),
            ..Default::default()
        };

        info!("Setting cache for profile: {}", params.username);
        self.redis_service
            .set_cached::<ProfileDetailsResponse>(&cache_key, &response, CACHE_TTL_SECONDS)
            .await?;

        info!(
            "Profile fetched successfully for username: {}",
            params.username
        );
        Ok(response)
    }

    pub async fn list_profiles(
        &self,
        params: &LeaderboardQuery,
    ) -> Result<LeaderboardResponse, ApiError> {
        info!("Listing profiles with params: {:?}", params);
        let tokens = self
            .token_service
            .list_token_picks(TokenQuery {
                get_all: Some(true),
                picked_after: Some(params.picked_after.clone()),
                group_ids: params.group_id.map(|id| vec![id]),
                ..Default::default()
            })
            .await?;

        let unique_users = tokens
            .0
            .iter()
            .map(|t| t.user.username.clone())
            .collect::<HashSet<_>>();
        info!("Found {} unique users", unique_users.len());
        let mut profiles = join_all(unique_users.iter().map(|username| {
            let query = ProfileQuery {
                username: username.clone(),
                picked_after: params.picked_after.clone(),
                group_id: params.group_id,
            };
            self.get_profile(query)
        }))
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;
        info!("Fetched {} profiles", profiles.len());

        profiles.sort_by(|a, b| match params.sort {
            Some(LeaderboardSort::PickReturns) => b
                .pick_summary
                .pick_returns
                .cmp(&a.pick_summary.pick_returns),
            Some(LeaderboardSort::HitRate) => b.pick_summary.hit_rate.cmp(&a.pick_summary.hit_rate),
            Some(LeaderboardSort::RealizedProfit) => b
                .pick_summary
                .realized_profit
                .cmp(&a.pick_summary.realized_profit),
            Some(LeaderboardSort::TotalPicks) => {
                b.pick_summary.total_picks.cmp(&a.pick_summary.total_picks)
            }
            _ => a.username.cmp(&b.username),
        });
        info!("Sorted profiles");

        Ok(LeaderboardResponse { profiles })
    }

    pub async fn get_user_picks_and_stats(
        &self,
        params: &ProfilePicksAndStatsQuery,
    ) -> Result<(Vec<TokenPickResponse>, UserStats), ApiError> {
        info!("Getting user picks and stats for {}", params.username);

        let paramsx = TokenQuery {
            username: Some(params.username.clone()),
            get_all: Some(true),
            picked_after: params.picked_after.clone(),
            group_ids: params.group_ids.clone(),
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
            ) - Decimal::one();
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
        Decimal::zero()
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
