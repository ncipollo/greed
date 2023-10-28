use crate::asset::AssetSymbol;
use crate::config::rules::{BuyRulesConfig, SellRulesConfig};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct AssetConfig {
    #[serde(default)]
    pub buy_rules: BuyRulesConfig,
    #[serde(default)]
    pub sell_rules: SellRulesConfig,
    pub symbol: AssetSymbol,
}
