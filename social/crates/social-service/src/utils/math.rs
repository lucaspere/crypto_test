use rust_decimal::{prelude::Zero, Decimal};

pub fn calculate_price_multiplier(
    initial_market_cap: &Decimal,
    current_market_cap: &Decimal,
) -> Decimal {
    if initial_market_cap.is_zero() {
        return Decimal::zero();
    }

    current_market_cap / initial_market_cap
}
