use crate::asset::AssetSymbol;
use crate::config::strategy::StrategyConfig;
use itertools::Itertools;

pub fn from_config(config: &StrategyConfig) -> Vec<AssetSymbol> {
    config
        .assets
        .iter()
        .map(|a| a.symbol.clone())
        .unique()
        .collect()
}

#[cfg(test)]
mod test {
    use crate::asset::AssetSymbol;
    use crate::config::asset::AssetConfig;
    use crate::config::strategy::StrategyConfig;
    use crate::strategy::symbols;

    #[test]
    fn from_config_empty() {
        let config: StrategyConfig = Default::default();
        let symbols = symbols::from_config(&config);
        assert_eq!(symbols, vec![])
    }

    #[test]
    fn from_config_no_duplicates() {
        let config: StrategyConfig = StrategyConfig {
            assets: vec![
                AssetConfig::with_symbol("VTI"),
                AssetConfig::with_symbol("SPDR"),
            ],
            ..Default::default()
        };
        let symbols = symbols::from_config(&config);
        let expected = vec![
            AssetSymbol::new("VTI"),
            AssetSymbol::new("SPDR"),
        ];
        assert_eq!(symbols, expected)
    }

    #[test]
    fn from_config_with_duplicates() {
        let config: StrategyConfig = StrategyConfig {
            assets: vec![
                AssetConfig::with_symbol("VTI"),
                AssetConfig::with_symbol("SPDR"),
                AssetConfig::with_symbol("VTI"),
            ],
            ..Default::default()
        };
        let symbols = symbols::from_config(&config);
        let expected = vec![
            AssetSymbol::new("VTI"),
            AssetSymbol::new("SPDR"),
        ];
        assert_eq!(symbols, expected)
    }
}
