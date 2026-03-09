use crate::lowercase_enum_display;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum OrderSide {
    #[default]
    Buy,
    Sell,
}

lowercase_enum_display!(OrderSide);

#[cfg(test)]
mod test {
    use crate::platform::order::side::OrderSide;

    #[test]
    fn default() {
        let side: OrderSide = Default::default();
        assert_eq!(side, OrderSide::Buy)
    }

    #[test]
    fn display() {
        let display = OrderSide::Buy.to_string();
        assert_eq!(display, "buy")
    }
}
