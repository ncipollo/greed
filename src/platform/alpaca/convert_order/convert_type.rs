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

impl From<OrderType> for Type {
    fn from(value: OrderType) -> Self {
        match value {
            OrderType::Market => Self::Market,
            OrderType::Limit => Self::Limit,
            OrderType::Stop => Self::Stop,
            OrderType::StopLimit => Self::StopLimit,
            OrderType::TrailingStop => Self::TrailingStop
        }
    }
}

#[cfg(test)]
mod test {
    use apca::api::v2::order::Type;
    use crate::assert;
    use crate::platform::order::order_type::OrderType;

    #[test]
    fn from() {
        assert::conversion(Type::Market, OrderType::Market);
        assert::conversion(Type::Limit, OrderType::Limit);
        assert::conversion(Type::Stop, OrderType::Stop);
        assert::conversion(Type::StopLimit, OrderType::StopLimit);
        assert::conversion(Type::TrailingStop, OrderType::TrailingStop);

    }

    #[test]
    fn into_alpaca() {
        assert::conversion(OrderType::Market, Type::Market);
        assert::conversion(OrderType::Limit, Type::Limit);
        assert::conversion(OrderType::Stop, Type::Stop);
        assert::conversion(OrderType::StopLimit, Type::StopLimit);
        assert::conversion(OrderType::TrailingStop, Type::TrailingStop);

    }
}
