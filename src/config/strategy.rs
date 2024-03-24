pub mod r#do;
pub mod r#for;
pub mod rule;
pub mod when;
pub mod median;

use itertools::Itertools;
use crate::asset::AssetSymbol;
use crate::config::strategy::rule::RuleConfig;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct StrategyConfig {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub buy: RuleConfig,
    #[serde(default)]
    pub sell: RuleConfig,
}

impl StrategyConfig {
    pub fn assets(&self) -> Vec<AssetSymbol> {
        let buy_assets = self.buy.assets();
        let sell_assets = self.sell.assets();
        return vec![buy_assets, sell_assets].into_iter()
            .flat_map(|thing| thing.into_iter())
            .unique()
            .collect::<Vec<_>>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::strategy::r#for::ForConfig;

    #[test]
    fn assets_buy_and_sell_rules() {
        let strategy = StrategyConfig {
            buy: buy_rules(),
            sell: sell_rules(),
            ..Default::default()
        };
        let expected: Vec<AssetSymbol> = vec!["BUY".into(), "SELL".into()];
        assert_eq!(expected, strategy.assets())
    }

    #[test]
    fn assets_only_buy_rules() {
        let strategy = StrategyConfig {
            buy: buy_rules(),
            ..Default::default()
        };
        let expected: Vec<AssetSymbol> = vec!["BUY".into()];
        assert_eq!(expected, strategy.assets())
    }

    #[test]
    fn assets_only_sell_rules() {
        let strategy = StrategyConfig {
            sell: sell_rules(),
            ..Default::default()
        };
        let expected: Vec<AssetSymbol> = vec!["SELL".into()];
        assert_eq!(expected, strategy.assets())
    }

    #[test]
    fn assets_only_uniques() {
        let strategy = StrategyConfig {
            buy: buy_rules(),
            sell: buy_rules(),
            ..Default::default()
        };
        let expected: Vec<AssetSymbol> = vec!["BUY".into()];
        assert_eq!(expected, strategy.assets())
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
