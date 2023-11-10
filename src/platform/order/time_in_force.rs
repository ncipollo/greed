#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TimeInForce {
    /// The order is good for the day, and it will be canceled
    /// automatically at the end of Regular Trading Hours if unfilled.
    Day,
    /// The order is only executed if the entire order quantity can
    /// be filled, otherwise the order is canceled.
    FillOrKill,
    /// The order requires all or part of the order to be executed
    /// immediately. Any unfilled portion of the order is canceled.
    ImmediateOrCancel,
    /// The order is good until canceled.
    UntilCanceled,
    /// This order is eligible to execute only in the market opening
    /// auction. Any unfilled orders after the open will be canceled.
    UntilMarketOpen,
    /// This order is eligible to execute only in the market closing
    /// auction. Any unfilled orders after the close will be canceled.
    UntilMarketClose,
}

impl Default for TimeInForce {
    fn default() -> Self {
        Self::Day
    }
}

#[cfg(test)]
mod test {
    use crate::platform::order::time_in_force::TimeInForce;

    #[test]
    fn default() {
        let status: TimeInForce = Default::default();
        assert_eq!(status, TimeInForce::Day)
    }
}