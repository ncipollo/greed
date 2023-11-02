use crate::asset::AssetSymbol;
use crate::config::rules::{BuyRulesConfig, SellRulesConfig};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct AssetConfig {
    #[serde(default)]
    pub buy_rules: BuyRulesConfig,
    #[serde(default)]
    pub sell_rules: SellRulesConfig,
    pub symbol: AssetSymbol,
}

impl AssetConfig {
    pub fn with_symbol(symbol: &str) -> Self {
        Self {
            symbol: AssetSymbol::new(symbol),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod test {
    use crate::asset::AssetSymbol;
    use crate::config::asset::AssetConfig;

    #[test]
    fn with_symbol() {
        let asset = AssetConfig::with_symbol("VTI");
        let expected = AssetConfig {
            symbol: AssetSymbol::new("VTI"),
            ..Default::default()
        };
        assert_eq!(asset, expected)
    }
}