use crate::config::asset::AssetConfig;
use crate::config::rules::{BuyRulesConfig, SellRulesConfig};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct StrategyConfig {
    #[serde(default)]
    pub assets: Vec<AssetConfig>,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub buy_rules: BuyRulesConfig,
    #[serde(default)]
    pub sell_rules: SellRulesConfig,
}
