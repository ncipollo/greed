use std::fmt::{Display, Formatter};
use crate::lowercase_enum_display;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OrderType {
    /// A market order.
    Market,
    /// A limit order.
    Limit,
    /// A stop on quote order.
    Stop,
    /// A stop limit order.
    StopLimit,
    /// A trailing stop order.
    TrailingStop,
}

impl Default for OrderType {
    fn default() -> Self {
        Self::Market
    }
}

lowercase_enum_display!(OrderType);

#[cfg(test)]
mod test {
    use crate::platform::order::order_type::OrderType;

    #[test]
    fn default() {
        let order_type: OrderType = Default::default();
        assert_eq!(order_type, OrderType::Market);
    }

    #[test]
    fn display() {
        let display = format!("{}, {}, {}, {}, {}", OrderType::Market,
                              OrderType::Limit,
                              OrderType::Stop,
                              OrderType::StopLimit,
                              OrderType::TrailingStop);
        assert_eq!(display, "market, limit, stop, stoplimit, trailingstop")
    }
}