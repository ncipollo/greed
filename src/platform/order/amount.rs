use num_decimal::Num;
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Amount {
    Quantity(Num),
    Notional(Num),
}

impl Display for Amount {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Amount::Quantity(value) => write!(f, "{} stock", value),
            Amount::Notional(value) => write!(f, "${} of stock", value)
        }
    }
}

#[cfg(test)]
mod test {
    use num_decimal::Num;
    use crate::platform::order::amount::Amount;

    #[test]
    fn display_quantity() {
        let display = Amount::Quantity(Num::from(5)).to_string();
        assert_eq!("5 stock", display)
    }

    #[test]
    fn display_notional() {
        let display = Amount::Notional(Num::from(10)).to_string();
        assert_eq!("$10 of stock", display)
    }
}
