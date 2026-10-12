#[cfg(test)]
use crate::config::simple::SimpleConfig;
#[cfg(test)]
use crate::config::Config;
#[cfg(test)]
use crate::db;
#[cfg(test)]
use duckdb::Connection;
#[cfg(test)]
use std::path::PathBuf;
#[cfg(test)]
use std::sync::{Arc, Mutex};

#[cfg(test)]
pub fn db_connection() -> Arc<Mutex<Connection>> {
    let connection = Connection::open_in_memory().expect("in-memory connection");
    db::migrations::apply(&connection).expect("migrations should apply");
    Arc::new(Mutex::new(connection))
}

#[cfg(test)]
pub async fn config(file: &str) -> Config {
    Config::from_path(path(file))
        .await
        .expect("config not found")
}

#[cfg(test)]
pub async fn simple_config(file: &str) -> Config {
    SimpleConfig::from_path(path(file))
        .await
        .expect("simple config not found")
        .into()
}

#[cfg(test)]
pub fn path(file: &str) -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    [manifest_dir, "src", "fixtures", file].iter().collect()
}
