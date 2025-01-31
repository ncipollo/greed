use crate::config::simple::tactic::SimpleTacticConfig;
use crate::config::simple::SimpleConfig;
use crate::error::GreedError;
use csv::Trim;
use std::path::Path;

pub async fn read_config<P: AsRef<Path>>(path: P) -> Result<SimpleConfig, GreedError> {
    let mut reader = csv::ReaderBuilder::new().trim(Trim::All).from_path(path)?;
    let mut tactics = Vec::new();
    for result in reader.deserialize() {
        let tactic: SimpleTacticConfig = result?;
        if !tactic.skip {
            tactics.push(tactic)
        }
    }
    Ok(SimpleConfig { tactics })
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
