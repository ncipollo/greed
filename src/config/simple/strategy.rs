use serde::{Deserialize, Serialize};

use crate::config::strategy::StrategyConfig;

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
struct SimpleStrategyConfig {
    strategies: Vec<StrategyConfig>,
}
