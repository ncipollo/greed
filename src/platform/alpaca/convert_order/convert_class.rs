use crate::platform::order::class::OrderClass;
use apca::api::v2::order::Class;

impl From<Class> for OrderClass {
    fn from(value: Class) -> Self {
        match value {
            Class::Simple => OrderClass::Simple,
            Class::Bracket => OrderClass::Bracket,
            Class::OneCancelsOther => OrderClass::OneCancelsOther,
            Class::OneTriggersOther => OrderClass::OneTriggersOther,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::platform::order::class::OrderClass;
    use apca::api::v2::order::Class;

    #[test]
    fn from() {
        assert_conversion(Class::Simple, OrderClass::Simple);
        assert_conversion(Class::Bracket, OrderClass::Bracket);
        assert_conversion(Class::OneCancelsOther, OrderClass::OneCancelsOther);
        assert_conversion(Class::OneTriggersOther, OrderClass::OneTriggersOther);
    }

    fn assert_conversion(alpaca_class: Class, expected: OrderClass) {
        let order_class: OrderClass = alpaca_class.into();
        assert_eq!(order_class, expected)
    }
}
