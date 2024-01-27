use serde::{Deserialize, Serialize};
use crate::asset::AssetSymbol;
use crate::config::strategy::r#do::DoConfig;
use crate::config::strategy::r#for::ForConfig;
use crate::config::strategy::when::WhenConfig;

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct RuleConfig {
    #[serde(rename = "for")]
    pub for_config: ForConfig,
    #[serde(rename = "when")]
    pub when_config: WhenConfig,
    #[serde(rename = "do")]
    pub do_config: DoConfig
}

impl RuleConfig {
    pub fn assets(&self) -> Vec<AssetSymbol> {
        self.for_config.assets()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assets() {
        let stock = AssetSymbol::new("VTI");
        let config = RuleConfig {
            for_config: ForConfig::Stock { stock: stock.clone() },
            ..Default::default()
        };
        assert_eq!(vec![stock.clone()], config.assets())
    }
}