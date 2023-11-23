use num_decimal::Num;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TakeProfit {
    Limit(Num)
}

impl Default for TakeProfit {
    fn default() -> Self {
        Self::Limit(Num::from(0))
    }
}

#[cfg(test)]
mod test {
    use num_decimal::Num;
    use crate::platform::request::take_profit::TakeProfit;

    #[test]
    fn default() {
        let take_profit: TakeProfit = Default::default();
        assert_eq!(take_profit, TakeProfit::Limit(Num::from(0)))
    }
}