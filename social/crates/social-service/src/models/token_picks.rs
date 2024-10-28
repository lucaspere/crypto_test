use super::tokens::Token;
use chrono::{DateTime, Duration, FixedOffset, Utc};
use rust_decimal::{prelude::FromPrimitive, Decimal};
use serde::{Deserialize, Serialize};
use sqlx::{types::Json, FromRow};
use uuid::Uuid;

pub const HIT_MULTIPLIER: u8 = 2;

#[derive(Clone, Debug, FromRow, Serialize, Deserialize, Default)]
pub struct TokenPick {
    pub id: Uuid,
    pub token: Json<Token>,
    pub user_id: Uuid,
    pub group_id: Uuid,
    pub call_type: String,
    pub price_at_call: Decimal,
    pub market_cap_at_call: Option<Decimal>,
    pub supply_at_call: Option<Decimal>,
    pub call_date: DateTime<FixedOffset>,
    pub created_at: DateTime<FixedOffset>,
    pub highest_market_cap: Option<Decimal>,
    pub hit_date: Option<DateTime<FixedOffset>>,
    pub points_awarded: bool,
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
        let fdv = self.market_cap_at_call.unwrap_or_default();
        let target_market_cap = fdv * Decimal::from(HIT_MULTIPLIER);

        if current_market_cap >= target_market_cap {
            self.hit_date = Some(Utc::now().into());
            true
        } else {
            false
        }
    }

    pub fn award_points(&mut self) -> bool {
        if self.points_awarded || self.hit_date.is_none() {
            return false;
        }

        let hit_date = self.hit_date.unwrap();
        let now: DateTime<FixedOffset> = Utc::now().into();

        if now - hit_date >= Duration::hours(24) {
            self.points_awarded = true;
            true
        } else {
            false
        }
    }
}

#[derive(Serialize)]
pub struct GetUserStatsParams {
    pub multiplier: Option<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

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
