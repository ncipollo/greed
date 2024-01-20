use crate::asset::AssetSymbol;
use chrono::{DateTime, Utc};
use num_decimal::Num;
use std::fmt::{Display, Formatter};
use once_cell::sync::Lazy;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Quote {
    pub time: DateTime<Utc>,
    /// The ask price.
    /// The ask is the lowest price where someone is willing to sell a share.
    pub ask_price: Num,
    /// The ask size.
    pub ask_size: u64,
    /// The bid price.
    /// The bid represents the highest price someone is willing to pay for a share.
    pub bid_price: Num,
    /// The bid size.
    pub bid_size: u64,
    pub symbol: AssetSymbol,
}

static ZERO_NUM: Lazy<Num> = Lazy::new(|| Num::from(0));

impl Quote {
    pub fn spread(&self) -> Num {
        &self.ask_price - &self.bid_price
    }

    pub fn spread_percent(&self) -> Num {
        if !self.valid_ask() {
            return Default::default()
        }

        let spread = self.spread();
        (spread / &self.ask_price) * 100
    }

    pub fn valid_ask(&self) -> bool {
        self.ask_price > *ZERO_NUM
    }

    pub fn valid_bid(&self) -> bool {
        self.bid_price > *ZERO_NUM
    }

    #[cfg(test)]
    pub fn fixture(symbol: AssetSymbol) -> Self {
        Self {
            time: Default::default(),
            ask_price: Num::from(200),
            ask_size: 1,
            bid_price: Num::from(100),
            bid_size: 1,
            symbol,
        }
    }
}

impl Display for Quote {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let ask = self.ask_price.to_f64().unwrap_or(0.0);
        let bid = self.bid_price.to_f64().unwrap_or(0.0);
        let spread = self.spread().to_f64().unwrap_or(0.0);
        let spread_percent = self.spread_percent().to_f64().unwrap_or(0.0);
        write!(
            f,
            "{} ask: {:.2}, bid: {:.2}, spread: {:.2} ({:.2}%)",
            self.symbol, ask, bid, spread, spread_percent
        )
    }
}

#[cfg(test)]
mod test {
    use crate::asset::AssetSymbol;
    use crate::platform::quote::Quote;
    use num_decimal::Num;
    use std::str::FromStr;

    #[test]
    fn display() {
        let quote = Quote {
            ask_price: Num::from_str("200.456").expect("failed to parse num"),
            bid_price: Num::from_str("100.123").expect("failed to parse num"),
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
        let expected = Num::from_str("0.05").expect("failed to parse num");
        assert_eq!(quote.spread(), expected)
    }

    #[test]
    fn spread_percent() {
        let quote = create_spread_quote();
        let expected = Num::from_str("0.5").expect("failed to parse num");
        assert_eq!(quote.spread_percent(), expected)
    }

    #[test]
    fn spread_percent_invalid_ask() {
        let mut quote = create_spread_quote();
        quote.ask_price = Default::default();

        let expected = Num::from_str("0.0").expect("failed to parse num");
        assert_eq!(quote.spread_percent(), expected)
    }

    #[test]
    fn valid_ask_invalid() {
        let quote = Quote {
            ask_price: Num::from_str("0.0").expect("failed to parse num"),
            ..Default::default()
        };
        assert!(!quote.valid_ask())
    }

    #[test]
    fn valid_ask_valid() {
        let quote = Quote {
            ask_price: Num::from_str("10.0").expect("failed to parse num"),
            ..Default::default()
        };
        assert!(quote.valid_ask())
    }

    #[test]
    fn valid_bid_invalid() {
        let quote = Quote {
            bid_price: Num::from_str("0.0").expect("failed to parse num"),
            ..Default::default()
        };
        assert!(!quote.valid_bid())
    }

    #[test]
    fn valid_bid_valid() {
        let quote = Quote {
            bid_price: Num::from_str("10.0").expect("failed to parse num"),
            ..Default::default()
        };
        assert!(quote.valid_bid())
    }

    fn create_spread_quote() -> Quote {
        Quote {
            ask_price: Num::from_str("10.0").expect("failed to parse num"),
            bid_price: Num::from_str("9.95").expect("failed to parse num"),
            ..Default::default()
        }
    }
}
