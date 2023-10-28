use std::path::Path;
use tokio::fs;
use crate::config::Config;
use crate::error::GreedError;

pub async fn read_config<P: AsRef<Path>>(path: P) -> Result<Config, GreedError> {
    let file_contents = fs::read_to_string(path).await?;
    Ok(toml::from_str(&file_contents)?)
}

#[cfg(test)]
mod test {
    use crate::config::reader::read_config;
    use crate::fixture;

    #[tokio::test]
    async fn read_config_not_found() {
        let path = fixture::path("bad_fixture.toml");
        read_config(path).await.expect_err("should have returned an error");
    }
}