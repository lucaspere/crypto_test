use crate::{
    repositories::token_repository::TokenPickRow,
    utils::{math::calculate_price_multiplier, time::TimePeriod},
};

use super::{
    groups::CreateOrUpdateGroup,
    tokens::Token,
    users::{User, UserResponse},
};
use chrono::{DateTime, FixedOffset, Utc};
use rust_decimal::{
    prelude::{FromPrimitive, ToPrimitive, Zero},
    Decimal,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::{IntoParams, ToSchema};

pub const HIT_MULTIPLIER: u8 = 2;

#[derive(Clone, Debug, FromRow, Serialize, Deserialize, Default)]
pub struct TokenPick {
    pub id: i64,
    #[sqlx(json)]
    pub token: Token,
    #[sqlx(json)]
    pub user: User,
    #[sqlx(json)]
    pub group: CreateOrUpdateGroup,
    pub telegram_message_id: Option<i64>,
    pub telegram_id: Option<i64>,
    pub price_at_call: Decimal,
    pub market_cap_at_call: Decimal,
    pub supply_at_call: Option<Decimal>,
    pub call_date: DateTime<FixedOffset>,
    pub highest_market_cap: Option<Decimal>,
    pub highest_multiplier: Option<Decimal>,
    pub hit_date: Option<DateTime<FixedOffset>>,
}

impl TokenPick {
    pub fn is_qualified(
        fdv: Decimal,
        liquidity: Option<Decimal>,
        volume_24h: Option<Decimal>,
    ) -> bool {
        if fdv <= Decimal::from(40_000) {
            return false;
        };

        match (liquidity, volume_24h) {
            (Some(liq), Some(vol)) => {
                if fdv < Decimal::from(1_000_000) {
                    liq >= vol * Decimal::from_f32(0.04).unwrap()
                } else {
                    liq >= Decimal::from(40_000)
                }
            }
            _ => false,
        }
    }

    pub fn check_for_hit(&mut self, current_market_cap: Decimal) -> bool {
        if self.hit_date.is_some() {
            return false;
        }
        let fdv = self.market_cap_at_call;
        let target_market_cap = fdv * Decimal::from(HIT_MULTIPLIER);

        if current_market_cap >= target_market_cap {
            self.hit_date = Some(Utc::now().into());
            true
        } else {
            false
        }
    }

    // pub fn award_points(&mut self) -> bool {
    //     if self.points_awarded || self.hit_date.is_none() {
    //         return false;
    //     }

    //     let hit_date = self.hit_date.unwrap();
    //     let now: DateTime<FixedOffset> = Utc::now().into();

    //     if now - hit_date >= Duration::hours(24) {
    //         self.points_awarded = true;
    //         true
    //     } else {
    //         false
    //     }
    // }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TokenPickResponse {
    /// The pick ID
    pub id: i64,
    /// The token info
    pub token: Token,
    /// The user ID
    pub user: UserResponse,
    /// The group ID
    pub group: CreateOrUpdateGroup,
    /// The market cap at the time the pick was made
    pub market_cap_at_call: Decimal,
    /// Date the pick was made
    pub call_date: DateTime<FixedOffset>,
    /// The highest market cap the pick has reached
    pub current_market_cap: Decimal,
    /// The current multiplier of the pick
    pub current_multiplier: f32,
    /// The highest market cap the pick has reached
    pub highest_mc_post_call: Option<Decimal>,
    /// The multiplier of the pick
    pub highest_mult_post_call: f32,
    /// Date the pick hit
    pub hit_date: Option<DateTime<FixedOffset>>,
}

impl From<TokenPick> for TokenPickResponse {
    fn from(pick: TokenPick) -> Self {
        let highest_mult_post_call = calculate_price_multiplier(
            &pick.market_cap_at_call,
            &pick.highest_market_cap.unwrap_or_default(),
        )
        .round_dp(2);
        let highest_mult_post_call = highest_mult_post_call
            .to_string()
            .parse::<f32>()
            .unwrap_or_default();

        let current_market_cap = pick.token.market_cap.unwrap_or_default();
        let current_multiplier =
            calculate_price_multiplier(&pick.market_cap_at_call, &current_market_cap)
                .round_dp(2)
                .to_f32()
                .unwrap_or_default();

        Self {
            token: pick.token,
            highest_mult_post_call,
            call_date: pick.call_date,
            group: pick.group,
            id: pick.id,
            user: pick.user.into(),
            highest_mc_post_call: pick.highest_market_cap.map(|mc| mc.round_dp(2)),
            hit_date: pick.hit_date,
            market_cap_at_call: pick.market_cap_at_call.round_dp(2),
            current_market_cap,
            current_multiplier,
            ..Default::default()
        }
    }
}

impl From<TokenPickRow> for TokenPickResponse {
    fn from(row: TokenPickRow) -> Self {
        let highest_mult_post_call = if let Some(highest_market_cap) = row.highest_market_cap {
            if highest_market_cap.is_zero() || row.market_cap_at_call.is_zero() {
                Decimal::zero()
            } else {
                highest_market_cap / row.market_cap_at_call
            }
        } else {
            Decimal::zero()
        }
        .round_dp(2);
        let highest_mult_post_call = highest_mult_post_call
            .to_string()
            .parse::<f32>()
            .unwrap_or_default();

        Self {
            highest_mult_post_call,
            call_date: row.call_date,
            id: row.id,
            highest_mc_post_call: row.highest_market_cap.map(|mc| mc.round_dp(2)),
            hit_date: row.hit_date,
            market_cap_at_call: row.market_cap_at_call.round_dp(2),
            token: Token {
                address: row.token_address,
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

#[derive(Deserialize, Default, IntoParams)]
pub struct ProfilePicksAndStatsQuery {
    pub username: String,
    pub multiplier: Option<u8>,
    pub picked_after: Option<TimePeriod>,
    pub group_ids: Option<Vec<i64>>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct TokenPicksGroup {
    pub address: String,
    #[sqlx(json)]
    pub picks: Vec<TokenPick>,
}
