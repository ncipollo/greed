use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, PartialEq)]
pub enum Amount {
    Quantity(f64),
    Notional(f64),
}

impl Default for Amount {
    fn default() -> Self {
        Self::Quantity(0.0)
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

impl Amount {
    pub fn is_empty(&self) -> bool {
        match self {
            Amount::Quantity(num) => *num == 0.0,
            Amount::Notional(num) => *num == 0.0,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::platform::order::amount::Amount;

    #[test]
    fn default() {
        let default: Amount = Default::default();
        assert_eq!(default, Amount::Quantity(0.0))
    }

    #[test]
    fn display_quantity() {
        let display = Amount::Quantity(5.0).to_string();
        assert_eq!("5.00 units", display)
    }

    #[test]
    fn display_notional() {
        let display = Amount::Notional(10.0).to_string();
        assert_eq!("$10.00", display)
    }

    #[test]
    fn is_empty_notational_empty() {
        let amount = Amount::Notional(0.0);
        assert!(amount.is_empty())
    }

    #[test]
    fn is_empty_notational_not_empty() {
        let amount = Amount::Notional(100.0);
        assert!(!amount.is_empty())
    }

    #[test]
    fn is_empty_quantity_empty() {
        let amount = Amount::Quantity(0.0);
        assert!(amount.is_empty())
    }

    #[test]
    fn is_empty_quantity_not_empty() {
        let amount = Amount::Quantity(100.0);
        assert!(!amount.is_empty())
    }
}
