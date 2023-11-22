use crate::platform::order::side::OrderSide;
use crate::platform::side::Side;

impl From<apca::api::v2::position::Side> for Side {
    fn from(value: apca::api::v2::position::Side) -> Self {
        match value {
            apca::api::v2::position::Side::Long => Self::Long,
            apca::api::v2::position::Side::Short => Self::Short,
        }
    }
}

impl From<apca::api::v2::order::Side> for OrderSide {
    fn from(value: apca::api::v2::order::Side) -> Self {
        match value {
            apca::api::v2::order::Side::Buy => Self::Buy,
            apca::api::v2::order::Side::Sell => Self::Sell,
        }
    }
}

impl From<OrderSide> for apca::api::v2::order::Side {
    fn from(value: OrderSide) -> apca::api::v2::order::Side {
        match value {
            OrderSide::Buy => Self::Buy,
            OrderSide::Sell => Self::Sell,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::platform::order::side::OrderSide;
    use crate::platform::side::Side;

    #[test]
    fn alpaca_order_into_buy() {
        let side: apca::api::v2::order::Side = OrderSide::Buy.into();
        assert_eq!(side, apca::api::v2::order::Side::Buy)
    }

    #[test]
    fn alpaca_order_into_sell() {
        let side: apca::api::v2::order::Side = OrderSide::Sell.into();
        assert_eq!(side, apca::api::v2::order::Side::Sell)
    }

    #[test]
    fn order_into_buy() {
        let side: OrderSide = apca::api::v2::order::Side::Buy.into();
        assert_eq!(side, OrderSide::Buy)
    }

    #[test]
    fn order_into_sell() {
        let side: OrderSide = apca::api::v2::order::Side::Sell.into();
        assert_eq!(side, OrderSide::Sell)
    }

    #[test]
    fn position_into_long() {
        let side: Side = apca::api::v2::position::Side::Long.into();
        assert_eq!(side, Side::Long)
    }

    #[test]
    fn position_into_short() {
        let side: Side = apca::api::v2::position::Side::Short.into();
        assert_eq!(side, Side::Short)
    }
}
