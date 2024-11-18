use rust_decimal::{prelude::Zero, Decimal};

pub fn calculate_return(market_cap_at_call: &Decimal, highest_market_cap: &Decimal) -> Decimal {
    if market_cap_at_call.is_zero() || highest_market_cap.is_zero() {
        Decimal::zero()
    } else {
        highest_market_cap / market_cap_at_call
    }
}
