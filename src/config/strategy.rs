use crate::config::tactic::TacticConfig;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct StrategyConfig {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub tactics: Vec<TacticConfig>,
}
