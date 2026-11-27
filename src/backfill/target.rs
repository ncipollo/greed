use crate::asset::AssetSymbol;
use crate::config::platform::PlatformType;
use crate::config::strategy::StrategyConfig;
use crate::error::GreedError;
use crate::strategy::reader::{read_config_from_path, read_tactics_from_config};
use itertools::Itertools;
use std::path::{Path, PathBuf};

/// What to backfill: explicit ticker symbols, or every symbol a greed config trades.
#[derive(Clone, Debug, PartialEq)]
pub enum BackfillTarget {
    Symbols(Vec<AssetSymbol>),
    Config(PathBuf),
}

/// The symbols (and platform) a config trades.
#[derive(Debug, PartialEq)]
pub struct ConfigTarget {
    pub platform: PlatformType,
    pub symbols: Vec<AssetSymbol>,
}

/// Collects every symbol named by a config's own tactics and by the strategy
/// files it points at. Agent strategies are skipped: an agent chooses its own
/// assets at runtime, so there's nothing to read ahead of time.
pub async fn target_from_config(config_path: &Path) -> Result<ConfigTarget, GreedError> {
    let config = read_config_from_path(config_path).await?;
    let mut symbols: Vec<AssetSymbol> = config.tactics.iter().flat_map(|t| t.assets()).collect();
    for strategy in &config.strategies {
        if let StrategyConfig::LocalFile { .. } = strategy {
            let tactics = read_tactics_from_config(config_path, strategy).await?;
            symbols.extend(tactics.iter().flat_map(|t| t.assets()));
        }
    }
    Ok(ConfigTarget {
        platform: config.platform,
        symbols: symbols.into_iter().unique().collect(),
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::fixture;

    fn symbols(names: &[&str]) -> Vec<AssetSymbol> {
        names.iter().map(|name| AssetSymbol::new(name)).collect()
    }

    #[tokio::test]
    async fn reads_symbols_from_top_level_tactics() {
        let target = target_from_config(&fixture::path("config_single_tactic.toml"))
            .await
            .expect("config should load");

        assert_eq!(target.symbols, symbols(&["VTI"]));
        assert_eq!(target.platform, PlatformType::Alpaca);
    }

    #[tokio::test]
    async fn reads_symbols_from_every_tactic_without_duplicates() {
        let target = target_from_config(&fixture::path("config_multi_tactic.toml"))
            .await
            .expect("config should load");

        assert_eq!(target.symbols, symbols(&["VTI", "UVXY"]));
    }

    #[tokio::test]
    async fn reads_symbols_from_strategy_files() {
        let target = target_from_config(&fixture::path("config_strategic.toml"))
            .await
            .expect("config should load");

        for expected in ["SPY", "VTI", "UVXY"] {
            assert!(
                target.symbols.contains(&AssetSymbol::new(expected)),
                "missing {expected} in {:?}",
                target.symbols
            );
        }
        let unique_count = target.symbols.iter().unique().count();
        assert_eq!(unique_count, target.symbols.len());
    }

    #[tokio::test]
    async fn reads_symbols_from_a_simple_csv_config() {
        let target = target_from_config(&fixture::path("simple_config_minimal.csv"))
            .await
            .expect("csv config should load");

        assert!(target.symbols.contains(&AssetSymbol::new("VTI")));
        assert!(target.symbols.contains(&AssetSymbol::new("SPY")));
    }

    #[tokio::test]
    async fn missing_config_is_an_error() {
        let result = target_from_config(&fixture::path("does_not_exist.toml")).await;

        assert!(result.is_err());
    }
}
