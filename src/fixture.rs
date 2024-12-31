#[cfg(test)]
use crate::config::simple::SimpleConfig;
#[cfg(test)]
use crate::config::Config;
#[cfg(test)]
use std::path::PathBuf;

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
