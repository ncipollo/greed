use crate::platform::order::class::OrderClass;
use apca::api::v2::order::Class;

impl From<Class> for OrderClass {
    fn from(value: Class) -> Self {
        match value {
            Class::Simple => OrderClass::Simple,
            Class::Bracket => OrderClass::Bracket,
            Class::OneCancelsOther => OrderClass::OneCancelsOther,
            Class::OneTriggersOther => OrderClass::OneTriggersOther,
            _ => panic!("unknown stop order class")
        }
    }
}

impl From<OrderClass> for Class {
    fn from(value: OrderClass) -> Self {
        match value {
            OrderClass::Simple => Class::Simple,
            OrderClass::Bracket => Class::Bracket,
            OrderClass::OneCancelsOther => Class::OneCancelsOther,
            OrderClass::OneTriggersOther => Class::OneTriggersOther,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::platform::order::class::OrderClass;
    use apca::api::v2::order::Class;
    use crate::assert;

    #[test]
    fn from() {
        assert::conversion(Class::Simple, OrderClass::Simple);
        assert::conversion(Class::Bracket, OrderClass::Bracket);
        assert::conversion(Class::OneCancelsOther, OrderClass::OneCancelsOther);
        assert::conversion(Class::OneTriggersOther, OrderClass::OneTriggersOther);
    }

    #[test]
    fn from_alpaca() {
        assert::conversion(OrderClass::Simple, Class::Simple);
        assert::conversion(OrderClass::Bracket, Class::Bracket);
        assert::conversion(OrderClass::OneCancelsOther, Class::OneCancelsOther);
        assert::conversion(OrderClass::OneTriggersOther, Class::OneTriggersOther);
    }
}
