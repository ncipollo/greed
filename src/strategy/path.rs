use crate::config::strategy::StrategyConfig;
use crate::error::GreedError;
use std::path::{Path, PathBuf};

pub fn path_for_config(
    config_path: &Path,
    strategy_config: &StrategyConfig,
) -> Result<PathBuf, GreedError> {
    match strategy_config {
        StrategyConfig::LocalFile { path, .. } => strategic_path(config_path, path),
    }
}

fn strategic_path(config_path: &Path, path: &str) -> Result<PathBuf, GreedError> {
    let path = Path::new(path);
    let directory = directory_from_path(config_path)?;
    let strategic_path = directory.join(path);
    Ok(strategic_path)
}

fn directory_from_path(path: &Path) -> Result<PathBuf, GreedError> {
    path.parent()
        .map(|p| p.to_path_buf())
        .ok_or(GreedError::new("config path was invalid"))
}

#[cfg(test)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture;

    #[test]
    fn path_for_config_valid_path() {
        let config_path = fixture::path("config_strategic.toml");
        let strategy_config = StrategyConfig::LocalFile {
            path: "strategy.csv".to_string(),
            properties: Default::default(),
        };
        let path = path_for_config(&config_path, &strategy_config).unwrap();

        assert_eq!(path, fixture::path("strategy.csv"))
    }
}
