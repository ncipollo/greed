use crate::config::platform::PlatformType;
use crate::config::simple::strategy::SimpleStrategyConfig;
use crate::config::Config;
use crate::error::GreedError;
use serde::{Deserialize, Serialize};
use std::path::Path;

mod reader;
mod strategy;

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct SimpleConfig {
    strategies: Vec<SimpleStrategyConfig>,
}

impl SimpleConfig {
    pub async fn from_path<P: AsRef<Path>>(path: P) -> Result<SimpleConfig, GreedError> {
        reader::read_config(path).await
    }
}

impl From<SimpleConfig> for Config {
    fn from(value: SimpleConfig) -> Self {
        Config {
            platform: PlatformType::Alpaca,
            strategies: value.strategies.into_iter().map(Into::into).collect(),
            interval: 5,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::config::platform::PlatformType;
    use crate::config::simple::reader::read_config;
    use crate::config::simple::strategy::SimpleStrategyConfig;
    use crate::config::simple::SimpleConfig;
    use crate::config::Config;
    use crate::fixture;

    #[tokio::test]
    async fn deserialize() {
        let path = fixture::path("simple_config_minimal.csv");
        let config = read_config(path).await.expect("config should load");
        let expected = SimpleConfig {
            strategies: vec![
                SimpleStrategyConfig {
                    asset: "VTI".into(),
                    amount: 50.0,
                    buy: Some(5.0),
                    sell: Some(1.0),
                    skip: false,
                },
                SimpleStrategyConfig {
                    asset: "SPY".into(),
                    amount: 25.0,
                    buy: Some(1.0),
                    sell: None,
                    skip: false,
                },
                SimpleStrategyConfig {
                    asset: "VEA".into(),
                    amount: 25.0,
                    buy: None,
                    sell: Some(2.0),
                    skip: false,
                },
            ],
        };
        assert_eq!(config, expected)
    }

    #[test]
    fn from_minimal_config() {
        let simple_config: SimpleConfig = Default::default();
        let expected = Config {
            platform: PlatformType::Alpaca,
            strategies: vec![],
            interval: 5,
        };
        assert_eq!(expected, Config::from(simple_config))
    }

    #[test]
    fn from_full_config() {
        let simple_strategy_1 = SimpleStrategyConfig {
            asset: "VTI".into(),
            amount: 50.0,
            buy: Some(5.0),
            sell: Some(1.0),
            skip: false,
        };
        let simple_strategy_2 = SimpleStrategyConfig {
            asset: "SPY".into(),
            amount: 25.0,
            buy: Some(1.0),
            sell: None,
            skip: false,
        };
        let expected = Config {
            platform: PlatformType::Alpaca,
            strategies: vec![
                simple_strategy_1.clone().into(),
                simple_strategy_2.clone().into(),
            ],
            interval: 5,
        };
        assert_eq!(
            expected,
            Config::from(SimpleConfig {
                strategies: vec![simple_strategy_1, simple_strategy_2],
            })
        )
    }
}
