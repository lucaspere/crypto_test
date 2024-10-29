use std::sync::Arc;

use rust_decimal::Decimal;

use crate::{
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
}

impl ProfileService {
    pub fn new(
        user_repository: Arc<UserRepository>,
        token_repository: Arc<TokenRepository>,
    ) -> Self {
        ProfileService {
            user_repository,
            token_repository,
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
        let user = self
            .user_repository
            .find_by_username(&username)
            .await?
            .ok_or(ApiError::UserNotFound)?;

        let picks = self
            .token_repository
            .find_token_picks_by_user_id(user.id, params)
            .await?;

        let total_picks = picks.len() as i32;
        let hits: Vec<&TokenPick> = picks.iter().filter(|p| p.hit_date.is_some()).collect();
        let total_hits = hits.len() as i32;
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
