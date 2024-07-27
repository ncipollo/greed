use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::config::simple::strategy::SimpleStrategyConfig;
use crate::error::GreedError;

mod reader;
mod strategy;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
struct SimpleConfig {
    strategies: Vec<SimpleStrategyConfig>,
}

impl SimpleConfig {
    pub async fn from_path<P: AsRef<Path>>(path: P) -> Result<SimpleConfig, GreedError> {
        reader::read_config(path).await
    }
}

#[cfg(test)]
mod test {
    use crate::config::simple::reader::read_config;
    use crate::config::simple::strategy::SimpleStrategyConfig;
    use crate::config::simple::SimpleConfig;
    use crate::fixture;

    #[tokio::test]
    async fn deserialize() {
        let path = fixture::path("simple_config_minimal.csv");
        let config = read_config(path).await.expect("config should load");
        let expected = SimpleConfig {
            strategies: vec![
                SimpleStrategyConfig {
                    asset: "VTI".into(),
                    buy: Some(5.0),
                    sell: Some(1.0),
                },
                SimpleStrategyConfig {
                    asset: "SPY".into(),
                    buy: Some(1.0),
                    sell: None,
                },
                SimpleStrategyConfig {
                    asset: "VEA".into(),
                    buy: None,
                    sell: Some(2.0),
                },
            ],
        };
        assert_eq!(config, expected)
    }
}
