use crate::config::strategy::StrategyConfig;
use crate::error::GreedError;
use std::path::{Path, PathBuf};

pub fn path_for_config(
    config_path: &str,
    strategy_config: &StrategyConfig,
) -> Result<String, GreedError> {
    match strategy_config {
        StrategyConfig::LocalFile { path, .. } => strategic_path(config_path, path),
    }
}

fn strategic_path(config_path: &str, path: &str) -> Result<String, GreedError> {
    let path = Path::new(path);
    let directory = directory_from_path(config_path)?;
    let strategic_path = directory.join(path);
    strategic_path
        .to_str()
        .ok_or(GreedError::new("strategic config path was invalid"))
        .map(|p| p.to_string())
}

fn directory_from_path(path: &str) -> Result<PathBuf, GreedError> {
    let path = Path::new(path);
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
        let fixture_path = fixture::path("config_strategic.toml");
        let config_path = fixture_path.to_str().unwrap();
        let strategy_config = StrategyConfig::LocalFile {
            path: "strategy.csv".to_string(),
            properties: Default::default(),
        };
        let path = path_for_config(&config_path, &strategy_config).unwrap();

        assert_eq!(path, fixture::path("strategy.csv").to_str().unwrap())
    }
}
