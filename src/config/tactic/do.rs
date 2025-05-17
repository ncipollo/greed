use crate::config::quote_fetcher_config::QuoteFetcherConfig;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(untagged)]
pub enum DoConfig {
    Buy { buy_percent: f64 },
    Nothing { nothing: bool },
    SellAll { sell_all: bool },
}

impl QuoteFetcherConfig for DoConfig {
    fn should_fetch_quotes(&self) -> bool {
        match self {
            DoConfig::Buy { .. } => true,
            &DoConfig::Nothing { .. } => false,
            DoConfig::SellAll { .. } => false,
        }
    }
}

impl Default for DoConfig {
    fn default() -> Self {
        Self::Nothing { nothing: true }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default() {
        assert_eq!(DoConfig::Nothing { nothing: true }, Default::default())
    }

    #[test]
    fn should_fetch_quotes_buy() {
        let config = DoConfig::Buy { buy_percent: 0.5 };
        assert!(config.should_fetch_quotes());
    }

    #[test]
    fn should_fetch_quotes_nothin() {
        let config = DoConfig::Nothing { nothing: true };
        assert!(!config.should_fetch_quotes());
    }

    #[test]
    fn should_fetch_quotes_sell_all() {
        let config = DoConfig::SellAll { sell_all: true };
        assert!(!config.should_fetch_quotes());
    }
}
