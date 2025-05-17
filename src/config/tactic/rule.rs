use crate::asset::AssetSymbol;
use crate::config::quote_fetcher_config::QuoteFetcherConfig;
use crate::config::tactic::r#do::DoConfig;
use crate::config::tactic::r#for::ForConfig;
use crate::config::tactic::when::WhenConfig;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct RuleConfig {
    #[serde(rename = "for", default)]
    pub for_config: ForConfig,
    #[serde(rename = "when", default)]
    pub when_config: WhenConfig,
    #[serde(rename = "do", default)]
    pub do_config: DoConfig,
}

impl RuleConfig {
    pub fn assets(&self) -> Vec<AssetSymbol> {
        self.for_config.assets()
    }
}

impl QuoteFetcherConfig for RuleConfig {
    fn should_fetch_quotes(&self) -> bool {
        self.for_config.should_fetch_quotes()
            || self.when_config.should_fetch_quotes()
            || self.do_config.should_fetch_quotes()
    }
}

#[cfg(test)]
mod tests {
    use crate::config::tactic::median::MedianPeriod;
    use super::*;

    #[test]
    fn assets() {
        let stock = AssetSymbol::new("VTI");
        let config = RuleConfig {
            for_config: ForConfig::Stock {
                stock: stock.clone(),
            },
            ..Default::default()
        };
        assert_eq!(vec![stock.clone()], config.assets())
    }

    #[test]
    fn should_fetch_quotes_when_below_median() {
        let config = RuleConfig {
            when_config: WhenConfig::BelowMedian {
                below_median_percent: 10.0,
                median_period: MedianPeriod::default(),
            },
            ..Default::default()
        };
        assert!(config.should_fetch_quotes());
    }

    #[test]
    fn should_fetch_quotes_when_buy() {
        let config = RuleConfig {
            do_config: DoConfig::Buy { buy_percent: 0.5 },
            ..Default::default()
        };
        assert!(config.should_fetch_quotes());
    }

    #[test]
    fn should_fetch_quotes_when_sell_all() {
        let config = RuleConfig {
            do_config: DoConfig::SellAll { sell_all: true },
            ..Default::default()
        };
        assert!(!config.should_fetch_quotes());
    }

    #[test]
    fn should_fetch_quotes_when_always() {
        let config = RuleConfig {
            when_config: WhenConfig::Always { always: true },
            do_config: DoConfig::SellAll { sell_all: true },
            ..Default::default()
        };
        assert!(!config.should_fetch_quotes());
    }
}
