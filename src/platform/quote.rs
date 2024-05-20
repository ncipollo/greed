use std::fmt::{Display, Formatter};

use chrono::{DateTime, Utc};

use crate::asset::AssetSymbol;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Quote {
    pub time: DateTime<Utc>,
    /// The ask price.
    /// The ask is the lowest price where someone is willing to sell a share.
    pub ask_price: f64,
    /// The ask size.
    pub ask_size: u64,
    /// The bid price.
    /// The bid represents the highest price someone is willing to pay for a share.
    pub bid_price: f64,
    /// The bid size.
    pub bid_size: u64,
    pub symbol: AssetSymbol,
}

impl Quote {
    pub fn spread(&self) -> f64 {
        &self.ask_price - &self.bid_price
    }

    pub fn spread_percent(&self) -> f64 {
        if !self.valid_ask() {
            return Default::default();
        }

        let spread = self.spread();
        (spread / &self.ask_price) * 100.0
    }

    pub fn valid_ask(&self) -> bool {
        self.ask_price > 0.0
    }

    pub fn valid_bid(&self) -> bool {
        self.bid_price > 0.0
    }

    #[cfg(test)]
    pub fn fixture(symbol: AssetSymbol) -> Self {
        Self {
            time: Default::default(),
            ask_price: 200.0,
            ask_size: 1,
            bid_price: 100.0,
            bid_size: 1,
            symbol,
        }
    }
}

impl Display for Quote {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let ask = self.ask_price;
        let bid = self.bid_price;
        let spread = self.spread();
        let spread_percent = self.spread_percent();
        write!(
            f,
            "{} ask: {:.2}, bid: {:.2}, spread: {:.2} ({:.2}%)",
            self.symbol, ask, bid, spread, spread_percent
        )
    }
}

#[cfg(test)]
mod test {
    use approx::{assert_relative_eq};
    use crate::asset::AssetSymbol;
    use crate::platform::quote::Quote;

    #[test]
    fn display() {
        let quote = Quote {
            ask_price: 200.456,
            bid_price: 100.123,
            symbol: AssetSymbol::new("vti"),
            ..Default::default()
        };
        let display = format!("{}", quote);
        let expected = "VTI ask: 200.46, bid: 100.12, spread: 100.33 (50.05%)";
        assert_eq!(display, expected)
    }

    #[test]
    fn spread() {
        let quote = create_spread_quote();
        let expected = 0.05;
        assert_relative_eq!(quote.spread(), expected, max_relative = 0.001);
    }

    #[test]
    fn spread_percent() {
        let quote = create_spread_quote();
        let expected = 0.5;
        assert_relative_eq!(quote.spread_percent(), expected, max_relative = 0.001)
    }

    #[test]
    fn spread_percent_invalid_ask() {
        let mut quote = create_spread_quote();
        quote.ask_price = Default::default();

        let expected = 0.0;
        assert_eq!(quote.spread_percent(), expected)
    }

    #[test]
    fn valid_ask_invalid() {
        let quote = Quote {
            ask_price: 0.0,
            ..Default::default()
        };
        assert!(!quote.valid_ask())
    }

    #[test]
    fn valid_ask_valid() {
        let quote = Quote {
            ask_price: 10.0,
            ..Default::default()
        };
        assert!(quote.valid_ask())
    }

    #[test]
    fn valid_bid_invalid() {
        let quote = Quote {
            bid_price: 0.0,
            ..Default::default()
        };
        assert!(!quote.valid_bid())
    }

    #[test]
    fn valid_bid_valid() {
        let quote = Quote {
            bid_price: 10.0,
            ..Default::default()
        };
        assert!(quote.valid_bid())
    }

    fn create_spread_quote() -> Quote {
        Quote {
            ask_price: 10.0,
            bid_price: 9.95,
            ..Default::default()
        }
    }
}
