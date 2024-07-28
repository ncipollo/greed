use crate::asset::AssetSymbol;
use crate::config::strategy::r#do::DoConfig;
use crate::config::strategy::r#for::ForConfig;
use crate::config::strategy::rule::RuleConfig;
use crate::config::strategy::when::WhenConfig;
use crate::config::strategy::StrategyConfig;
use serde::{de, Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct SimpleStrategyConfig {
    /// The Asset to buy and sell.
    pub asset: AssetSymbol,
    /// The amount as a percent of your total portfolio to buy.
    pub amount: f64,
    /// The percent below the monthly median the price of the asset must hit for us to buy.
    pub buy: Option<f64>,
    /// The gains percent we must hit before we will sell.
    pub sell: Option<f64>,
    /// Set to true in to skip this row in the config.
    #[serde(default, deserialize_with = "deserialize_bool")]
    pub skip: bool,
}

fn deserialize_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: de::Deserializer<'de>,
{
    let field: &str = de::Deserialize::deserialize(deserializer)?;
    let lower_field = field.to_lowercase();
    match lower_field.as_str() {
        "true" | "yes" | "1" => Ok(true),
        _ => Ok(false),
    }
}

impl From<SimpleStrategyConfig> for StrategyConfig {
    fn from(value: SimpleStrategyConfig) -> Self {
        StrategyConfig {
            name: value.asset.symbol.clone(),
            buy: buy_rules(&value),
            sell: sell_rules(&value),
        }
    }
}

fn buy_rules(simple_config: &SimpleStrategyConfig) -> RuleConfig {
    if let Some(buy) = simple_config.buy {
        RuleConfig {
            for_config: ForConfig::Stock {
                stock: simple_config.asset.clone(),
            },
            when_config: WhenConfig::BelowMedian {
                below_median_percent: buy,
                median_period: Default::default(),
            },
            do_config: DoConfig::Buy {
                buy_percent: simple_config.amount,
            },
        }
    } else {
        RuleConfig::default()
    }
}

fn sell_rules(simple_config: &SimpleStrategyConfig) -> RuleConfig {
    if let Some(sell) = simple_config.sell {
        RuleConfig {
            for_config: ForConfig::Stock {
                stock: simple_config.asset.clone(),
            },
            when_config: WhenConfig::GainAbove {
                gain_above_percent: sell,
            },
            do_config: DoConfig::SellAll { sell_all: true },
        }
    } else {
        RuleConfig::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_minimal_config() {
        let simple_config = SimpleStrategyConfig {
            asset: AssetSymbol::new("VTI"),
            ..Default::default()
        };
        let expected = StrategyConfig {
            name: "VTI".into(),
            buy: RuleConfig::default(),
            sell: RuleConfig::default(),
        };
        assert_eq!(expected, StrategyConfig::from(simple_config))
    }

    #[test]
    fn from_full_config() {
        let simple_config = SimpleStrategyConfig {
            asset: AssetSymbol::new("VTI"),
            amount: 0.5,
            buy: Some(0.1),
            sell: Some(0.2),
            skip: true,
        };
        let expected = StrategyConfig {
            name: "VTI".into(),
            buy: RuleConfig {
                for_config: ForConfig::Stock {
                    stock: "VTI".into(),
                },
                when_config: WhenConfig::BelowMedian {
                    below_median_percent: 0.1,
                    median_period: Default::default(),
                },
                do_config: DoConfig::Buy { buy_percent: 0.5 },
            },
            sell: RuleConfig {
                for_config: ForConfig::Stock {
                    stock: "VTI".into(),
                },
                when_config: WhenConfig::GainAbove {
                    gain_above_percent: 0.2,
                },
                do_config: DoConfig::SellAll { sell_all: true },
            },
        };
        assert_eq!(expected, StrategyConfig::from(simple_config))
    }
}
