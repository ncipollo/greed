const EXAMPLE: &str = r#"[[strategies]]
name = "Core"
portfolio_percent = 75.0
path = "strategy.toml"

[[strategies]]
name = "Simple"
portfolio_percent = 15.0
path = "simple.csv"

[[strategies]]
name = "Agent"
portfolio_percent = 10.0
agent_path = "agent.toml"
"#;

pub fn render() -> String {
    format!(
        r#"GREED: STRATEGIES

Strategies compose several tactic sources into one config and give each a
share of the portfolio. They are [[strategies]] entries in the main config
(see `greed info config`).

KEYS
  name               string  label for the strategy (default empty)
  portfolio_percent  number  share of the portfolio for this strategy (default 100)
  path               string  tactic file: TOML (`greed init strategy`) or .csv
                             (`greed info config-simple`); chosen by extension
  agent_path         string  AI agent config (`greed info agent`)

Exactly one of `path` or `agent_path` selects the strategy type. Relative
paths are resolved against the main config file.

EXAMPLE
{EXAMPLE}"#
    )
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::strategy::StrategyConfig;
    use crate::config::Config;

    #[test]
    fn example_parses_as_strategies() {
        let config: Config = toml::from_str(EXAMPLE).expect("example should parse");
        assert_eq!(config.strategies.len(), 3);
        assert!(matches!(
            config.strategies[0],
            StrategyConfig::LocalFile { .. }
        ));
        assert!(matches!(config.strategies[2], StrategyConfig::Agent { .. }));
    }
}
