pub mod migrations;
pub mod portfolio_status;
pub mod stock_quotes;

use crate::error::GreedError;
use duckdb::Connection;
use std::path::{Path, PathBuf};

const DEFAULT_DB_FILENAME: &str = "greed.duckdb";

pub fn default_path() -> PathBuf {
    PathBuf::from(DEFAULT_DB_FILENAME)
}

pub fn bootstrap_at(path: &Path) -> Result<Connection, GreedError> {
    let connection = Connection::open(path)?;
    migrations::apply(&connection)?;
    Ok(connection)
}

pub fn path_for_run(config_path: &Path, override_path: Option<&Path>) -> PathBuf {
    if let Some(override_path) = override_path {
        return override_path.to_path_buf();
    }
    match config_path.parent() {
        Some(parent) => parent.join(DEFAULT_DB_FILENAME),
        None => default_path(),
    }
}

pub fn path_for_override(override_path: Option<&Path>) -> PathBuf {
    override_path
        .map(Path::to_path_buf)
        .unwrap_or_else(default_path)
}

#[cfg(test)]
mod test {
    use crate::db::{bootstrap_at, default_path, path_for_override, path_for_run};
    use std::path::{Path, PathBuf};
    use uuid::Uuid;

    #[test]
    fn default_path_is_well_known_filename() {
        assert_eq!(default_path(), PathBuf::from("greed.duckdb"));
    }

    #[test]
    fn path_for_run_prefers_override_when_given() {
        let resolved = path_for_run(
            Path::new("/configs/my_config.toml"),
            Some(Path::new("/elsewhere/custom.duckdb")),
        );
        assert_eq!(resolved, PathBuf::from("/elsewhere/custom.duckdb"));
    }

    #[test]
    fn path_for_run_colocates_with_config_parent_directory() {
        let resolved = path_for_run(Path::new("/configs/my_config.toml"), None);
        assert_eq!(resolved, PathBuf::from("/configs/greed.duckdb"));
    }

    #[test]
    fn path_for_run_falls_back_to_default_path_when_config_has_no_parent() {
        let resolved = path_for_run(Path::new("/"), None);
        assert_eq!(resolved, default_path());
    }

    #[test]
    fn path_for_override_prefers_override_when_given() {
        let resolved = path_for_override(Some(Path::new("/elsewhere/custom.duckdb")));
        assert_eq!(resolved, PathBuf::from("/elsewhere/custom.duckdb"));
    }

    #[test]
    fn path_for_override_defaults_to_well_known_filename_when_omitted() {
        assert_eq!(path_for_override(None), default_path());
    }

    #[test]
    fn bootstrap_at_creates_the_database_file() {
        let path = temp_db_path();

        bootstrap_at(&path).expect("bootstrap should succeed");

        assert!(path.exists());
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn bootstrap_at_is_idempotent_across_restarts() {
        let path = temp_db_path();

        bootstrap_at(&path).expect("first bootstrap should succeed");
        bootstrap_at(&path).expect("second bootstrap should succeed");

        std::fs::remove_file(&path).ok();
    }

    fn temp_db_path() -> PathBuf {
        std::env::temp_dir().join(format!("greed_test_{}.duckdb", Uuid::new_v4()))
    }
}
