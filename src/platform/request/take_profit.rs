use num_decimal::Num;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TakeProfit {
    Limit(Num),
}

impl Default for TakeProfit {
    fn default() -> Self {
        Self::Limit(Num::from(0))
    }
}

#[cfg(test)]
mod test {
    use crate::platform::request::take_profit::TakeProfit;
    use num_decimal::Num;

    #[test]
    fn default() {
        let take_profit: TakeProfit = Default::default();
        assert_eq!(take_profit, TakeProfit::Limit(Num::from(0)))
    }
}
