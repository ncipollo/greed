pub mod migrations;

use crate::error::GreedError;
use duckdb::Connection;
use std::path::{Path, PathBuf};

const DEFAULT_DB_FILENAME: &str = "greed.duckdb";

pub fn default_path() -> PathBuf {
    PathBuf::from(DEFAULT_DB_FILENAME)
}

pub fn bootstrap() -> Result<Connection, GreedError> {
    bootstrap_at(&default_path())
}

pub fn bootstrap_at(path: &Path) -> Result<Connection, GreedError> {
    let connection = Connection::open(path)?;
    migrations::apply(&connection)?;
    Ok(connection)
}

#[cfg(test)]
mod test {
    use crate::db::{bootstrap_at, default_path};
    use std::path::PathBuf;
    use uuid::Uuid;

    #[test]
    fn default_path_is_well_known_filename() {
        assert_eq!(default_path(), PathBuf::from("greed.duckdb"));
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
