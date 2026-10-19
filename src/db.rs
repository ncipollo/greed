pub mod migrations;
pub mod portfolio_status;
mod retry;
pub mod stock_quotes;

use crate::error::GreedError;
use duckdb::{AccessMode, Config, Connection};
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

pub async fn bootstrap_at_with_retry(path: &Path) -> Result<Connection, GreedError> {
    retry::with_backoff(|| bootstrap_at(path)).await
}

/// Opens a connection for writing without re-checking the schema. Callers must
/// have already ensured the schema exists via `bootstrap_at`, since a brief
/// per-tick write window shouldn't pay for a migration check on every open.
pub fn open_for_write(path: &Path) -> Result<Connection, GreedError> {
    Connection::open(path).map_err(GreedError::from)
}

pub async fn open_for_write_with_retry(path: &Path) -> Result<Connection, GreedError> {
    retry::with_backoff(|| open_for_write(path)).await
}

fn open_read_only_now(path: &Path) -> Result<Connection, GreedError> {
    let config = Config::default().access_mode(AccessMode::ReadOnly)?;
    Connection::open_with_flags(path, config).map_err(GreedError::from)
}

/// Opens a read-only connection with a bounded retry on lock contention.
/// Returns `Ok(None)` when the database file doesn't exist yet, since a
/// read-only open can't create the schema the way a read-write open would.
pub async fn open_read_only_with_retry(path: &Path) -> Result<Option<Connection>, GreedError> {
    if !path.exists() {
        return Ok(None);
    }
    let connection = retry::with_backoff(|| open_read_only_now(path)).await?;
    Ok(Some(connection))
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
    use crate::db::{
        bootstrap_at, default_path, open_for_write, open_for_write_with_retry,
        open_read_only_with_retry, path_for_override, path_for_run,
    };
    use std::path::{Path, PathBuf};
    use std::time::Duration;
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

    #[tokio::test]
    async fn open_for_write_with_retry_succeeds_once_lock_is_released() {
        let path = temp_db_path();
        bootstrap_at(&path).expect("bootstrap should succeed");
        let holder = open_for_write(&path).expect("initial open should succeed");

        let path_clone = path.clone();
        let retry_task = tokio::spawn(async move { open_for_write_with_retry(&path_clone).await });

        tokio::time::sleep(Duration::from_millis(60)).await;
        drop(holder);

        let result = retry_task.await.expect("retry task should not panic");
        assert!(result.is_ok());

        std::fs::remove_file(&path).ok();
    }

    #[tokio::test]
    async fn open_read_only_with_retry_returns_none_for_missing_file() {
        let path = temp_db_path();

        let result = open_read_only_with_retry(&path)
            .await
            .expect("missing file should not error");

        assert!(result.is_none());
        assert!(!path.exists());
    }

    #[tokio::test]
    async fn open_read_only_with_retry_opens_existing_file() {
        let path = temp_db_path();
        bootstrap_at(&path).expect("bootstrap should succeed");

        let result = open_read_only_with_retry(&path)
            .await
            .expect("read-only open should succeed");

        assert!(result.is_some());
        std::fs::remove_file(&path).ok();
    }

    fn temp_db_path() -> PathBuf {
        std::env::temp_dir().join(format!("greed_test_{}.duckdb", Uuid::new_v4()))
    }
}
