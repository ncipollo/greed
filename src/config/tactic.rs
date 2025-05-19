pub mod r#do;
pub mod r#for;
pub mod median;
pub mod rule;
pub mod when;

use crate::asset::AssetSymbol;
use crate::config::quote_fetcher_config::QuoteFetcherConfig;
use crate::config::tactic::rule::RuleConfig;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct TacticConfig {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub buy: RuleConfig,
    #[serde(default)]
    pub sell: RuleConfig,
}

impl TacticConfig {
    pub fn assets(&self) -> Vec<AssetSymbol> {
        let buy_assets = self.buy.assets();
        let sell_assets = self.sell.assets();
        return vec![buy_assets, sell_assets]
            .into_iter()
            .flat_map(|thing| thing.into_iter())
            .unique()
            .collect::<Vec<_>>();
    }
}

impl QuoteFetcherConfig for TacticConfig {
    fn should_fetch_quotes(&self) -> bool {
        self.buy.should_fetch_quotes() || self.sell.should_fetch_quotes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::tactic::r#for::ForConfig;
    use crate::config::tactic::r#do::DoConfig;
    use crate::config::tactic::when::WhenConfig;
    use crate::config::tactic::median::MedianPeriod;

    #[test]
    fn assets_buy_and_sell_rules() {
        let tactic = TacticConfig {
            buy: buy_rules(),
            sell: sell_rules(),
            ..Default::default()
        };
        let expected: Vec<AssetSymbol> = vec!["BUY".into(), "SELL".into()];
        assert_eq!(expected, tactic.assets())
    }

    #[test]
    fn assets_only_buy_rules() {
        let tactic = TacticConfig {
            buy: buy_rules(),
            ..Default::default()
        };
        let expected: Vec<AssetSymbol> = vec!["BUY".into()];
        assert_eq!(expected, tactic.assets())
    }

    #[test]
    fn assets_only_sell_rules() {
        let tactic = TacticConfig {
            sell: sell_rules(),
            ..Default::default()
        };
        let expected: Vec<AssetSymbol> = vec!["SELL".into()];
        assert_eq!(expected, tactic.assets())
    }

    #[test]
    fn assets_only_uniques() {
        let tactic = TacticConfig {
            buy: buy_rules(),
            sell: buy_rules(),
            ..Default::default()
        };
        let expected: Vec<AssetSymbol> = vec!["BUY".into()];
        assert_eq!(expected, tactic.assets())
    }

    #[test]
    fn should_fetch_quotes_buy_requires_quotes() {
        let tactic = TacticConfig {
            buy: RuleConfig {
                do_config: DoConfig::Buy { buy_percent: 0.5 },
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(tactic.should_fetch_quotes());
    }

    #[test]
    fn should_fetch_quotes_sell_requires_quotes() {
        let tactic = TacticConfig {
            sell: RuleConfig {
                when_config: WhenConfig::BelowMedian {
                    below_median_percent: 10.0,
                    median_period: MedianPeriod::default(),
                },
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(tactic.should_fetch_quotes());
    }

    #[test]
    fn should_fetch_quotes_neither_requires_quotes() {
        let tactic = TacticConfig {
            buy: RuleConfig {
                do_config: DoConfig::SellAll { sell_all: true },
                ..Default::default()
            },
            sell: RuleConfig {
                when_config: WhenConfig::Always { always: true },
                do_config: DoConfig::SellAll { sell_all: true },
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(!tactic.should_fetch_quotes());
    }

    fn buy_rules() -> RuleConfig {
        RuleConfig {
            for_config: ForConfig::Stock {
                stock: "BUY".into(),
            },
            ..Default::default()
        }
    }

    fn sell_rules() -> RuleConfig {
        RuleConfig {
            for_config: ForConfig::Stock {
                stock: "SELL".into(),
            },
            ..Default::default()
        }
    }
}
