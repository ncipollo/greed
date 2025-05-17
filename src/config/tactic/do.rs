use serde::{Deserialize, Serialize};
use crate::config::quote_fetcher_config::QuoteFetcherConfig;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(untagged)]
pub enum DoConfig {
    Buy { buy_percent: f64 },
    SellAll { sell_all: bool },
}

impl QuoteFetcherConfig for DoConfig {
    fn should_fetch_quotes(&self) -> bool {
        false
    }
}

impl Default for DoConfig {
    fn default() -> Self {
        Self::Buy { buy_percent: 0.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default() {
        assert_eq!(DoConfig::Buy { buy_percent: 0.0 }, Default::default())
    }

    #[test]
    fn should_fetch_quotes_buy() {
        let config = DoConfig::Buy { buy_percent: 0.5 };
        assert!(!config.should_fetch_quotes());
    }

    #[test]
    fn should_fetch_quotes_sell_all() {
        let config = DoConfig::SellAll { sell_all: true };
        assert!(!config.should_fetch_quotes());
    }
}
