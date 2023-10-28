use crate::config::platform::PlatformType;
use crate::config::reader::read_config;
use crate::config::strategy::StrategyConfig;
use crate::error::GreedError;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub mod asset;
pub mod platform;
pub mod reader;
pub mod rules;
pub mod strategy;

#[derive(Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Config {
    #[serde(default)]
    platform: PlatformType,
    #[serde(default)]
    strategies: Vec<StrategyConfig>,
}

impl Config {
    pub async fn from_path<P: AsRef<Path>>(path: P) -> Result<Config, GreedError> {
        read_config(path).await
    }
}

#[cfg(test)]
mod test {
    use crate::asset::AssetSymbol;
    use crate::config::asset::AssetConfig;
    use crate::config::platform::PlatformType;
    use crate::config::rules::target::PortfolioTargetRule;
    use crate::config::rules::BuyRulesConfig;
    use crate::config::strategy::StrategyConfig;
    use crate::config::Config;
    use crate::fixture;

    #[test]
    fn default() {
        let default = Config::default();
        let expected = Config {
            platform: PlatformType::Alpaca,
            strategies: vec![],
        };

        assert_eq!(default, expected)
    }

    #[tokio::test]
    async fn deserialize_minimal_config() {
        let path = fixture::path("config_minimal.toml");
        let config = Config::from_path(path)
            .await
            .expect("minimal config not found");

        let expected = Config {
            platform: PlatformType::Alpaca,
            strategies: vec![],
        };
        assert_eq!(config, expected)
    }

    #[tokio::test]
    async fn deserialize_minimal_asset_config() {
        let path = fixture::path("config_minimal_asset.toml");
        let config = Config::from_path(path)
            .await
            .expect("minimal config not found");

        let expected = Config {
            platform: PlatformType::Alpaca,
            strategies: vec![StrategyConfig {
                assets: vec![AssetConfig {
                    buy_rules: Default::default(),
                    sell_rules: Default::default(),
                    symbol: AssetSymbol::new("VTI"),
                }],
                ..Default::default()
            }],
        };
        assert_eq!(config, expected)
    }

    #[tokio::test]
    async fn deserialize_multi_strategy_config() {
        let path = fixture::path("config_multi_strategy.toml");
        let config = Config::from_path(path)
            .await
            .expect("minimal config not found");

        let expected = Config {
            platform: PlatformType::Alpaca,
            strategies: vec![
                StrategyConfig {
                    assets: vec![
                        AssetConfig {
                            buy_rules: BuyRulesConfig {
                                portfolio_target: Some(PortfolioTargetRule { percent: 25.0 }),
                            },
                            sell_rules: Default::default(),
                            symbol: AssetSymbol::new("VTI"),
                        },
                        AssetConfig {
                            buy_rules: BuyRulesConfig {
                                portfolio_target: Some(PortfolioTargetRule { percent: 15.0 }),
                            },
                            sell_rules: Default::default(),
                            symbol: AssetSymbol::new("VEA"),
                        },
                    ],
                    buy_rules: BuyRulesConfig {
                        portfolio_target: Some(PortfolioTargetRule { percent: 50.0 }),
                    },
                    name: "ETFs".to_string(),
                    ..Default::default()
                },
                StrategyConfig {
                    assets: vec![
                        AssetConfig {
                            buy_rules: Default::default(),
                            sell_rules: Default::default(),
                            symbol: AssetSymbol::new("VIXY"),
                        }
                    ],
                    name: "Chaos".to_string(),
                    ..Default::default()
                },
            ],
        };
        assert_eq!(config, expected)
    }
}
