use num_decimal::Num;
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Amount {
    Quantity(Num),
    Notional(Num),
}

impl Default for Amount {
    fn default() -> Self {
        Self::Quantity(Num::from(0))
    }
}

impl Display for Amount {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Amount::Quantity(value) => write!(f, "{:.2} units", value),
            Amount::Notional(value) => write!(f, "${:.2}", value),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::platform::order::amount::Amount;
    use num_decimal::Num;

    #[test]
    fn default() {
        let default: Amount = Default::default();
        assert_eq!(default, Amount::Quantity(Num::from(0)))
    }

    #[test]
    fn display_quantity() {
        let display = Amount::Quantity(Num::from(5)).to_string();
        assert_eq!("5.00 units", display)
    }

    #[test]
    fn display_notional() {
        let display = Amount::Notional(Num::from(10)).to_string();
        assert_eq!("$10.00", display)
    }
}
