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

#[cfg(test)]
mod test {
    use crate::platform::order::order_type::OrderType;

    #[test]
    fn default() {
        let order_type: OrderType = Default::default();
        assert_eq!(order_type, OrderType::Market);
    }
}