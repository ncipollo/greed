use std::path::Path;

use crate::config::simple::strategy::SimpleStrategyConfig;
use crate::config::simple::SimpleConfig;
use crate::error::GreedError;

pub async fn read_config<P: AsRef<Path>>(path: P) -> Result<SimpleConfig, GreedError> {
    let mut reader = csv::Reader::from_path(path)?;
    let mut strategies = Vec::new();
    for result in reader.deserialize() {
        let strategy: SimpleStrategyConfig = result?;
        strategies.push(strategy)
    }
    Ok(SimpleConfig { strategies })
}

#[cfg(test)]
mod test {
    use crate::config::reader::read_config;
    use crate::fixture;

    #[tokio::test]
    async fn read_config_not_found() {
        let path = fixture::path("bad_fixture.toml");
        read_config(path)
            .await
            .expect_err("should have returned an error");
    }
}
