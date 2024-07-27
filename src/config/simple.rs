use serde::{Deserialize, Serialize};

use crate::config::simple::strategy::SimpleStrategyConfig;

mod reader;
mod strategy;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
struct SimpleConfig {
    strategies: Vec<SimpleStrategyConfig>,
}
