use crate::config::simple::SimpleConfig;
use crate::config::strategy::StrategyConfig;
use crate::config::tactic::TacticConfig;
use crate::config::Config;
use crate::error::GreedError;
use crate::strategy::path::path_for_config;
use std::ffi::OsStr;
use std::path::Path;

pub async fn ready_tactics_from_config(
    config_path: &Path,
    strategy_config: &StrategyConfig,
) -> Result<Vec<TacticConfig>, GreedError> {
    let path = path_for_config(config_path, strategy_config)?;
    let config = read_config_from_path(&path).await?;
    Ok(config.tactics)
}

async fn read_config_from_path(path: &Path) -> Result<Config, GreedError> {
    let ext = path.extension();
    if Some(OsStr::new("csv")) == ext {
        let simple_config = SimpleConfig::from_path(&path).await?;
        Ok(simple_config.into())
    } else {
        Config::from_path(&path).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture;

    #[tokio::test]
    async fn ready_tactics_from_config_csv_config() {
        let config_path = fixture::path("config_strategic.toml");
        let strategy_config = StrategyConfig::LocalFile {
            path: "simple_config_minimal.csv".to_string(),
            properties: Default::default(),
        };
        let tactics = ready_tactics_from_config(&config_path, &strategy_config)
            .await
            .expect("should have read tactics from config");
        let expected_config = fixture::simple_config("simple_config_minimal.csv").await;
        assert_eq!(tactics, expected_config.tactics);
    }

    #[tokio::test]
    async fn ready_tactics_from_config_toml_config() {
        let config_path = fixture::path("config_strategic.toml");
        let strategy_config = StrategyConfig::LocalFile {
            path: "config_single_tactic.toml".to_string(),
            properties: Default::default(),
        };
        let tactics = ready_tactics_from_config(&config_path, &strategy_config)
            .await
            .expect("should have read tactics from config");
        let expected_config = fixture::config("config_single_tactic.toml").await;
        assert_eq!(tactics, expected_config.tactics);
    }

    #[tokio::test]
    async fn ready_tactics_from_config_invalid_path() {
        let config_path = fixture::path("non_existent_config.toml");
        let strategy_config = StrategyConfig::LocalFile {
            path: "non_existent_file.toml".to_string(),
            properties: Default::default(),
        };
        let result = ready_tactics_from_config(&config_path, &strategy_config).await;
        assert!(
            result.is_err(),
            "should have returned an error for invalid path"
        );
    }
}
