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
    use crate::assert;

    #[test]
    fn from() {
        assert::conversion(Class::Simple, OrderClass::Simple);
        assert::conversion(Class::Bracket, OrderClass::Bracket);
        assert::conversion(Class::OneCancelsOther, OrderClass::OneCancelsOther);
        assert::conversion(Class::OneTriggersOther, OrderClass::OneTriggersOther);
    }
}
