use super::tokens::Token;
use chrono::{DateTime, FixedOffset, Utc};
use rust_decimal::{prelude::FromPrimitive, Decimal};
use serde::{Deserialize, Serialize};
use sqlx::{types::Json, FromRow};
use utoipa::ToSchema;
use uuid::Uuid;

pub const HIT_MULTIPLIER: u8 = 2;

#[derive(Clone, Debug, FromRow, Serialize, Deserialize, Default)]
pub struct TokenPick {
    pub id: i64,
    pub token: Json<Token>,
    pub user_id: Uuid,
    pub group_id: i64,
    pub telegram_message_id: Option<i64>,
    pub price_at_call: Decimal,
    pub market_cap_at_call: Decimal,
    pub supply_at_call: Option<Decimal>,
    pub call_date: DateTime<FixedOffset>,
    pub highest_market_cap: Option<Decimal>,
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
pub struct TokenPickResponse {
    /// The pick ID
    pub id: i64,
    /// The token info
    pub token: Token,
    /// The user ID
    pub user_id: Uuid,
    /// The group ID
    pub group_id: i64,
    /// The market cap at the time the pick was made
    pub market_cap_at_call: Decimal,
    /// Date the pick was made
    pub call_date: DateTime<FixedOffset>,
    /// The highest market cap the pick has reached
    pub current_market_cap: Decimal,
    /// The current multiplier of the pick
    pub current_multiplier: f32,
    /// The highest market cap the pick has reached
    pub highest_market_cap: Option<Decimal>,
    /// The multiplier of the pick
    pub higher_multiplier: f32,
    /// Date the pick hit
    pub hit_date: Option<DateTime<FixedOffset>>,
}

impl From<TokenPick> for TokenPickResponse {
    fn from(pick: TokenPick) -> Self {
        let higher_multiplier =
            (pick.highest_market_cap.unwrap_or_default() / pick.market_cap_at_call).floor();
        let higher_multiplier = higher_multiplier
            .to_string()
            .parse::<f32>()
            .unwrap_or_default();

        Self {
            token: pick.token.0,
            higher_multiplier,
            call_date: pick.call_date,
            group_id: pick.group_id,
            id: pick.id,
            user_id: pick.user_id,
            highest_market_cap: pick.highest_market_cap,
            hit_date: pick.hit_date,
            market_cap_at_call: pick.market_cap_at_call,
            ..Default::default()
        }
    }
}

#[derive(Deserialize, Default)]
pub struct ProfilePicksAndStatsQuery {
    pub username: String,
    pub multiplier: Option<u8>,
}

#[cfg(test)]
mod tests {
    // #[test]
    // fn test_check_for_hit() {
    //     let mut pick = TokenPick::new();
    //     pick.market_cap_at_call = Some(Decimal::from_f32(100_000_000.0).unwrap());
    //     assert_eq!(
    //         pick.check_for_hit(Decimal::from_f32(200_000_000.0).unwrap()),
    //         true
    //     );
    //     assert_eq!(
    //         pick.check_for_hit(Decimal::from_f32(199_000_000.0).unwrap()),
    //         false
    //     );
    // }
}
