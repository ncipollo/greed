use crate::platform::request::take_profit::TakeProfit;

impl From<apca::api::v2::order::TakeProfit> for TakeProfit {
    fn from(value: apca::api::v2::order::TakeProfit) -> Self {
        match value {
            apca::api::v2::order::TakeProfit::Limit(price) => Self::Limit(price),
        }
    }
}

#[cfg(test)]
mod test {
    use num_decimal::Num;
    use crate::assert;
    use crate::platform::request::take_profit::TakeProfit;

    #[test]
    fn into() {
        assert::conversion(
            apca::api::v2::order::TakeProfit::Limit(Num::from(42)),
            TakeProfit::Limit(Num::from(42))
        );
    }
}