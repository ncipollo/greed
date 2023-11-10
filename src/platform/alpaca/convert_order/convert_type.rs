use apca::api::v2::order::Type;
use crate::platform::order::order_type::OrderType;

impl From<Type> for OrderType {
    fn from(value: Type) -> Self {
        match value {
            Type::Market => OrderType::Market,
            Type::Limit => OrderType::Limit,
            Type::Stop => OrderType::Stop,
            Type::StopLimit => OrderType::StopLimit,
            Type::TrailingStop => OrderType::TrailingStop
        }
    }
}

#[cfg(test)]
mod test {
    use apca::api::v2::order::Type;
    use crate::platform::order::order_type::OrderType;

    #[test]
    fn from() {
        assert_conversion(Type::Market, OrderType::Market);
        assert_conversion(Type::Limit, OrderType::Limit);
        assert_conversion(Type::Stop, OrderType::Stop);
        assert_conversion(Type::StopLimit, OrderType::StopLimit);
        assert_conversion(Type::TrailingStop, OrderType::TrailingStop);

    }

    fn assert_conversion(alpaca_type: Type, expected: OrderType) {
        let order_type: OrderType = alpaca_type.into();
        assert_eq!(order_type, expected)
    }
}
