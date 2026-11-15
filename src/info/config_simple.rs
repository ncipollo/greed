const EXAMPLE: &str = "asset,amount,buy,sell,skip
VTI,50.0,5.0,1.0,false
SPY,25.0,1.0,,false
VEA,25.0,,2.0,true
";

pub fn render() -> String {
    format!(
        r#"GREED: SIMPLE CONFIG (CSV)

A CSV file (extension .csv) can be passed to `greed run` / `greed server`
instead of a TOML config, or referenced as a strategy `path`. Each row is one
asset tactic.

COLUMNS (header row required)
  asset   symbol to trade, e.g. VTI
  amount  percent of the total portfolio to buy
  buy     buy when price is this percent below the monthly median (blank = never buy)
  sell    sell everything once gains exceed this percent (blank = never sell)
  skip    true/yes/1 (case-insensitive) skips the row; anything else keeps it

When loaded directly as the main config it behaves like a TOML config with
platform "alpaca" and interval 5 seconds.

EXAMPLE
{EXAMPLE}"#
    )
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::simple::SimpleConfig;
    use crate::config::Config;

    #[tokio::test]
    async fn example_parses_as_simple_config() {
        let path =
            std::env::temp_dir().join(format!("greed-info-simple-{}.csv", std::process::id()));
        tokio::fs::write(&path, EXAMPLE).await.unwrap();
        let simple = SimpleConfig::from_path(&path).await;
        let _ = tokio::fs::remove_file(&path).await;
        let config: Config = simple.expect("example should parse").into();
        // The skipped VEA row is dropped.
        assert_eq!(config.tactics.len(), 2);
        assert_eq!(config.interval, 5);
    }
}
