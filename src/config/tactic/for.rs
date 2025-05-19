use crate::asset::AssetSymbol;
use crate::config::quote_fetcher_config::QuoteFetcherConfig;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(untagged)]
pub enum ForConfig {
    AnyOf { any_of: Vec<AssetSymbol> },
    Nothing { nothing: bool },
    Stock { stock: AssetSymbol },
}

impl ForConfig {
    pub fn assets(&self) -> Vec<AssetSymbol> {
        match self {
            ForConfig::AnyOf { any_of } => any_of.clone(),
            ForConfig::Nothing { .. } => vec![],
            ForConfig::Stock { stock } => vec![stock.clone()],
        }
    }
}

impl Default for ForConfig {
    fn default() -> Self {
        Self::Nothing { nothing: true }
    }
}

impl QuoteFetcherConfig for ForConfig {
    fn should_fetch_quotes(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assets_nothing() {
        assert_eq!(
            Vec::<AssetSymbol>::new(),
            ForConfig::Nothing { nothing: true }.assets()
        )
    }

    #[test]
    fn assets_stock() {
        let stock = AssetSymbol::new("VTI");
        assert_eq!(
            vec![stock.clone()],
            ForConfig::Stock {
                stock: stock.clone()
            }
            .assets()
        )
    }

    #[test]
    fn default() {
        assert_eq!(ForConfig::Nothing { nothing: true }, Default::default())
    }

    #[test]
    fn should_fetch_quotes_any_of() {
        let any_of = ForConfig::AnyOf {
            any_of: vec![AssetSymbol::new("VTI")],
        };
        assert!(!any_of.should_fetch_quotes());
    }

    #[test]
    fn should_fetch_quotes_nothing() {
        let nothing = ForConfig::Nothing { nothing: true };
        assert!(!nothing.should_fetch_quotes());
    }

    #[test]
    fn should_fetch_quotes_stock() {
        let stock = ForConfig::Stock {
            stock: AssetSymbol::new("VTI"),
        };
        assert!(!stock.should_fetch_quotes());
    }
}
