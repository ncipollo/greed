#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrderClass {
    /// Any non-bracket order (i.e., regular market, limit, or stop loss
    /// orders).
    Simple,
    /// A bracket order is a chain of three orders that can be used to manage your
    /// position entry and exit. It is a common use case of an
    /// one-triggers & one-cancels-other order.
    Bracket,
    /// A One-cancels-other is a set of two orders with the same side
    /// (buy/buy or sell/sell) and currently only exit order is supported.
    /// Such an order can be used to add two legs to an already filled
    /// order.
    OneCancelsOther,
    /// A one-triggers-other order that can either have a take-profit or
    /// stop-loss leg set. It essentially attached a single leg to an
    /// entry order.
    OneTriggersOther,
}

impl Default for OrderClass {
    fn default() -> Self {
        Self::Simple
    }
}

#[cfg(test)]
mod test {
    use crate::platform::order::class::OrderClass;

    #[test]
    fn default() {
        let class: OrderClass = Default::default();
        assert_eq!(class, OrderClass::Simple)
    }
}