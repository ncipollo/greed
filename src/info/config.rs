const EXAMPLE: &str = r#"platform = "alpaca"
interval = 60

[[strategies]]
name = "Core"
portfolio_percent = 75.0
path = "strategy.toml"

[[strategies]]
name = "Agent"
portfolio_percent = 25.0
agent_path = "agent.toml"

[[tactics]]
name = "ETF"

[tactics.buy]
for = { stock = "VTI" }
when = { below_median_percent = 5.0, median_period = "month" }
do = { buy_percent = 10.0 }

[tactics.sell]
for = { stock = "VTI" }
when = { gain_above_percent = 5.0 }
do = { sell_all = true }
"#;

pub fn render() -> String {
    format!(
        r#"GREED: CONFIG (greed.toml)

The main config is a TOML file passed to `greed run <PATH>` / `greed server <PATH>`.
Generate a starter with `greed init greed`.

KEYS (all optional)
  platform    string   trading platform; only "alpaca" (default)
  interval    integer  seconds between trading loop ticks (default 60)
  strategies  array    [[strategies]] entries, see `greed info strategy`
  tactics     array    [[tactics]] entries, see `greed info tactic`

Top-level `tactics` and `strategies` can be combined. A config with only
`tactics` is the simplest form. Use strategies to split the portfolio between
several tactic files and/or AI agents.

For a CSV alternative see `greed info config-simple`.

EXAMPLE
{EXAMPLE}"#
    )
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::Config;

    #[test]
    fn example_parses_as_config() {
        let config: Config = toml::from_str(EXAMPLE).expect("example should parse");
        assert_eq!(config.interval, 60);
        assert_eq!(config.strategies.len(), 2);
        assert_eq!(config.tactics.len(), 1);
    }
}
