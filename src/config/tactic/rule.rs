use crate::asset::AssetSymbol;
use crate::config::tactic::r#do::DoConfig;
use crate::config::tactic::r#for::ForConfig;
use crate::config::tactic::when::WhenConfig;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct RuleConfig {
    #[serde(rename = "for", default)]
    pub for_config: ForConfig,
    #[serde(rename = "when", default)]
    pub when_config: WhenConfig,
    #[serde(rename = "do", default)]
    pub do_config: DoConfig,
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
            for_config: ForConfig::Stock {
                stock: stock.clone(),
            },
            ..Default::default()
        };
        assert_eq!(vec![stock.clone()], config.assets())
    }
}
