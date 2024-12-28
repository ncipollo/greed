use crate::config::platform::PlatformType;
use crate::config::reader::read_config;
use crate::config::tactic::TacticConfig;
use crate::error::GreedError;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub mod platform;
pub mod reader;
pub mod simple;
pub mod strategy;
pub mod tactic;

#[derive(Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Config {
    #[serde(default)]
    pub platform: PlatformType,
    #[serde(default)]
    pub tactics: Vec<TacticConfig>,
    #[serde(default = "default_interval")]
    pub interval: u64,
}

fn default_interval() -> u64 {
    60
}

impl Config {
    pub async fn from_path<P: AsRef<Path>>(path: P) -> Result<Config, GreedError> {
        read_config(path).await
    }
}

#[cfg(test)]
mod test {
    use crate::config::platform::PlatformType;
    use crate::config::tactic::median::MedianPeriod;
    use crate::config::tactic::r#do::DoConfig;
    use crate::config::tactic::r#for::ForConfig;
    use crate::config::tactic::rule::RuleConfig;
    use crate::config::tactic::when::WhenConfig;
    use crate::config::tactic::TacticConfig;
    use crate::config::Config;
    use crate::fixture;

    #[test]
    fn default() {
        let default = Config::default();
        let expected = Config {
            platform: PlatformType::Alpaca,
            tactics: vec![],
            interval: 0,
        };

        assert_eq!(default, expected)
    }

    #[tokio::test]
    async fn deserialize_minimal_config() {
        let config = fixture::config("config_minimal.toml").await;
        let expected = Config {
            platform: PlatformType::Alpaca,
            tactics: vec![],
            interval: 60,
        };
        assert_eq!(expected, config)
    }

    #[tokio::test]
    async fn deserialize_single_tactic() {
        let config = fixture::config("config_single_tactic.toml").await;
        let expected = Config {
            platform: PlatformType::Alpaca,
            tactics: vec![TacticConfig {
                name: "ETF".to_string(),
                buy: RuleConfig {
                    for_config: ForConfig::Stock {
                        stock: "VTI".into(),
                    },
                    when_config: WhenConfig::BelowMedian {
                        below_median_percent: 5.0,
                        median_period: Default::default(),
                    },
                    do_config: DoConfig::Buy { buy_percent: 10.0 },
                },
                sell: RuleConfig {
                    for_config: ForConfig::Stock {
                        stock: "VTI".into(),
                    },
                    when_config: WhenConfig::GainAbove {
                        gain_above_percent: 5.0,
                    },
                    do_config: DoConfig::SellAll { sell_all: true },
                },
            }],
            interval: 300,
        };
        assert_eq!(expected, config)
    }

    #[tokio::test]
    async fn deserialize_multi_tactic_config() {
        let config = fixture::config("config_multi_tactic.toml").await;
        let expected = Config {
            platform: PlatformType::Alpaca,
            tactics: vec![
                TacticConfig {
                    name: "ETF".to_string(),
                    buy: RuleConfig {
                        for_config: ForConfig::Stock {
                            stock: "VTI".into(),
                        },
                        when_config: WhenConfig::BelowMedian {
                            below_median_percent: 5.0,
                            median_period: Default::default(),
                        },
                        do_config: DoConfig::Buy { buy_percent: 10.0 },
                    },
                    sell: RuleConfig {
                        for_config: ForConfig::Stock {
                            stock: "VTI".into(),
                        },
                        when_config: WhenConfig::GainAbove {
                            gain_above_percent: 5.0,
                        },
                        do_config: DoConfig::SellAll { sell_all: true },
                    },
                },
                TacticConfig {
                    name: "Chaos".to_string(),
                    buy: RuleConfig {
                        for_config: ForConfig::Stock {
                            stock: "UVXY".into(),
                        },
                        when_config: WhenConfig::BelowMedian {
                            below_median_percent: 2.0,
                            median_period: MedianPeriod::Week,
                        },
                        do_config: DoConfig::Buy { buy_percent: 5.0 },
                    },
                    sell: RuleConfig {
                        for_config: ForConfig::Stock {
                            stock: "UVXY".into(),
                        },
                        when_config: WhenConfig::GainAbove {
                            gain_above_percent: 3.0,
                        },
                        do_config: DoConfig::SellAll { sell_all: true },
                    },
                },
            ],
            interval: 300,
        };
        assert_eq!(expected, config)
    }
}
