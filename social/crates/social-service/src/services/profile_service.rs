use std::{collections::HashMap, collections::HashSet, sync::Arc};

use futures::future::join_all;
use rayon::slice::ParallelSliceMut;
use rust_decimal::{
    prelude::{FromPrimitive, Zero},
    Decimal,
};
use tracing::{error, info};
use uuid::Uuid;

use crate::{
    apis::{
        api_models::{
            query::{
                PickLeaderboardSort, ProfileLeaderboardQuery, ProfileLeaderboardSort, TokenQuery,
            },
            response::LeaderboardResponse,
        },
        profile_handlers::ProfileQuery,
    },
    external_services::{
        birdeye::BirdeyeService, cielo::CieloService, rust_monorepo::RustMonorepoService,
        usergate::UserGateService,
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

use super::{redis_service::RedisService, s3_service::S3Service, token_service::TokenService};

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
    usergate_service: Arc<UserGateService>,
    s3_service: Arc<S3Service>,
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
        usergate_service: Arc<UserGateService>,
        s3_service: Arc<S3Service>,
    ) -> Self {
        ProfileService {
            user_repository,
            token_repository,
            rust_monorepo_service,
            birdeye_service,
            redis_service,
            token_service,
            cielo_service,
            usergate_service,
            s3_service,
        }
    }

    pub async fn get_profile(
        &self,
        params: ProfileQuery,
        user_id: Option<Uuid>,
    ) -> Result<ProfileDetailsResponse, ApiError> {
        let user = self
            .user_repository
            .find_by_username(&params.username)
            .await?
            .ok_or(ApiError::UserNotFound)?;

        info!(
            "User found, fetching user picks and stats for username: {}",
            params.username
        );

        let is_following = match user_id {
            Some(id) if id == user.id => None,
            Some(id) => self.user_repository.is_following(id, user.id).await.ok(),
            None => None,
        };

        let (_, stats) = self
            .get_user_picks_and_stats(&ProfilePicksAndStatsQuery {
                username: params.username.clone(),
                picked_after: Some(params.picked_after.clone()),
                multiplier: None,
                group_ids: params.group_ids.clone(),
            })
            .await?;

        let response = ProfileDetailsResponse {
            id: user.id,
            username: params.username.clone(),
            name: Some(params.username.clone()),
            avatar_url: user.image_uri,
            bio: user.bio,
            pick_summary: ProfilePickSummary::from(stats),
            is_following,
            ..Default::default()
        };

        info!(
            "Profile fetched successfully for username: {}",
            params.username
        );
        Ok(response)
    }

    pub async fn list_profiles(
        &self,
        params: &ProfileLeaderboardQuery,
    ) -> Result<LeaderboardResponse, ApiError> {
        info!("Listing profiles with params: {:?}", params);
        let cache_key = format!(
            "leaderboard:{}:{}{}:{}",
            params.picked_after.to_string(),
            params
                .group_ids
                .clone()
                .map_or(String::new(), |ids| format!(
                    ":{}",
                    ids.iter()
                        .map(|id| id.to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                )),
            params.following,
            params
                .username
                .clone()
                .map_or(String::new(), |username| format!(":{}", username))
        );
        if let Some(cached_response) = self
            .redis_service
            .get_cached::<LeaderboardResponse>(&cache_key)
            .await?
        {
            return Ok(cached_response);
        }

        let tokens = self
            .token_service
            .list_token_picks(
                TokenQuery {
                    get_all: Some(true),
                    picked_after: Some(params.picked_after.clone()),
                    group_ids: params.group_ids.clone(),
                    following: params.following.then_some(true),
                    username: params.username.clone(),
                    ..Default::default()
                },
                None,
            )
            .await?;

        let unique_users = tokens
            .0
            .iter()
            .map(|t| t.user.as_ref().map(|u| u.username.clone()))
            .collect::<HashSet<_>>();
        info!("Found {} unique users", unique_users.len());
        let mut profiles = join_all(unique_users.iter().map(|username| {
            let query = ProfileQuery {
                username: username.clone().unwrap_or_default(),
                picked_after: params.picked_after.clone(),
                group_ids: params.group_ids.clone(),
            };
            self.get_profile(query, params.user_id)
        }))
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;
        info!("Fetched {} profiles", profiles.len());

        let sort_key = |profile: &ProfileDetailsResponse| match params.sort {
            Some(ProfileLeaderboardSort::PickReturns) => profile.pick_summary.pick_returns,
            Some(ProfileLeaderboardSort::HitRate) => profile.pick_summary.hit_rate,
            Some(ProfileLeaderboardSort::RealizedProfit) => profile.pick_summary.realized_profit,
            Some(ProfileLeaderboardSort::TotalPicks) => {
                Decimal::from(profile.pick_summary.total_picks)
            }
            Some(ProfileLeaderboardSort::AverageReturn) => profile.pick_summary.average_pick_return,
            Some(ProfileLeaderboardSort::GreatestHits) => profile.pick_summary.best_pick.multiplier,
            _ => Decimal::ZERO,
        };

        profiles.par_sort_by(|a, b| {
            if params.sort.is_none() {
                a.username.cmp(&b.username)
            } else {
                sort_key(b)
                    .cmp(&sort_key(a))
                    .then(a.username.cmp(&b.username))
            }
        });

        info!("Sorted profiles");
        let response = LeaderboardResponse { profiles };
        self.redis_service
            .set_cached::<LeaderboardResponse>(&cache_key, &response, CACHE_TTL_SECONDS)
            .await?;
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
            picked_after: params.picked_after.clone(),
            group_ids: params.group_ids.clone(),
            order_by: Some(PickLeaderboardSort::Reached),
            order_direction: Some("desc".to_string()),
            ..Default::default()
        };

        let (picks, _) = self
            .token_service
            .list_token_picks(paramsx, Some(false))
            .await?;

        if picks.is_empty() {
            info!("No picks found for user {}", params.username);
            return Ok((vec![], UserStats::default()));
        }

        let first_picks = picks.iter().fold(
            HashMap::<String, &TokenPickResponse>::new(),
            |mut acc, pick| {
                acc.entry(pick.token.address.clone()).or_insert(pick);
                acc
            },
        );

        let mut total_returns = Decimal::ZERO;
        let mut best_pick = None::<BestPick>;
        let mut hits_2x = 0;
        let mut hit_returns = Decimal::ZERO;
        for pick in first_picks.values() {
            if pick.highest_mult_post_call >= 2.0 {
                hits_2x += 1;
                hit_returns += Decimal::from_f32(pick.highest_mult_post_call).unwrap_or_default();
            }

            let current_return = calculate_price_multiplier(
                &pick.market_cap_at_call,
                &pick.highest_mc_post_call.unwrap_or_default(),
            );
            total_returns += current_return;

            let new_best = BestPick {
                token_symbol: pick.token.symbol.clone(),
                token_address: pick.token.address.clone(),
                multiplier: current_return,
                logo_uri: pick.token.logo_uri.clone(),
            };

            best_pick = match best_pick {
                Some(b) if current_return > b.multiplier => Some(new_best),
                None => Some(new_best),
                b => b,
            };
        }

        let total_picks = first_picks.len() as i32;
        let total_hits = first_picks
            .values()
            .filter(|p| p.hit_date.is_some())
            .count() as i32;

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
        let user = self
            .user_repository
            .find_by_username(&params.username)
            .await?
            .ok_or(ApiError::UserNotFound)?;

        info!(
            "User found, fetching user picks and stats for username: {}",
            params.username
        );
        let mut realized_profit = Decimal::ZERO;
        if let Some(wallet) = user.wallet_addresses.as_ref().and_then(|wa| {
            wa.iter()
                .filter(|w| w.address.is_some())
                .find(|w| w.chain == Some(Chain::Solana.to_string()))
        }) {
            let realized_pnl_usd = self
                .cielo_service
                .get_wallet_stats(wallet.address.as_ref().unwrap(), None)
                .await?;
            realized_profit = realized_pnl_usd.realized_pnl_usd.round_dp(2);
        }
        let usergate_stats = self
            .usergate_service
            .get_user_trading_stats(&user.id.to_string())
            .await
            .map_err(|e| {
                error!("Error fetching usergate stats: {:?}", e);
                e
            })
            .unwrap_or_default();

        let average_hit_return = if total_hits > 0 {
            hit_returns / Decimal::from(total_hits)
        } else {
            Decimal::ZERO
        };
        let total_busts = self.token_repository.count_busts(&user.id).await?;
        let stats = UserStats {
            total_picks,
            hit_rate: hit_rate.round_dp(2),
            pick_returns: total_returns.round_dp(2),
            average_pick_return: average_pick_return.round_dp(2),
            realized_profit,
            total_volume_traded: usergate_stats.trading_volume_usd.round_dp(2),
            hits: total_hits,
            misses: total_picks - total_hits,
            best_pick: best_pick.unwrap_or_default(),
            average_hit_return,
            total_busts,
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

fn calculate_price_multiplier(
    market_cap_at_call: &Decimal,
    highest_market_cap: &Decimal,
) -> Decimal {
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
