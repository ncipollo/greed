const EXAMPLE: &str = r#"[[tactics]]
name = "VTI"

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
    let body = r#"GREED: TACTICS

A tactic is a [[tactics]] entry (in the main config or a strategy `path` file)
with a `name` and optional `buy` and `sell` rules. Each rule has three parts:
`for` (which assets), `when` (the condition), `do` (the action).

KEYS
  name  string  label for the tactic
  buy   rule    [tactics.buy]
  sell  rule    [tactics.sell]
  rule keys: for, when, do (inline tables)

FOR (default: nothing)
  { stock = "VTI" }                 a single symbol
  { any_of = ["VTI", "SPY"] }       any of these symbols
  { all_other_positions = true }    every other position held
  { nothing = true }                matches nothing

WHEN (default: always)
  { always = true }
  { never = true }
  { below_median_percent = 5.0, median_period = "month" }
        price is this percent below the median; median_period is
        "day", "week" or "month" (default "month")
  { gain_above_percent = 5.0 }      position gain exceeds this percent
  { all_off = [ {...}, {...} ] }
        combine other `when` conditions (the key is spelled `all_off`)

DO (default: nothing)
  { buy_percent = 10.0 }            buy this percent of the portfolio
  { sell_all = true }               sell the whole position
  { nothing = true }

Unknown or mistyped keys silently fall back to the default variant, so
double-check spelling.

EXAMPLE
"#;
    format!("{body}{EXAMPLE}")
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::tactic::r#do::DoConfig;
    use crate::config::tactic::r#for::ForConfig;
    use crate::config::tactic::when::WhenConfig;
    use crate::config::Config;

    #[test]
    fn example_parses_with_expected_variants() {
        let config: Config = toml::from_str(EXAMPLE).expect("example should parse");
        let tactic = &config.tactics[0];
        assert!(matches!(tactic.buy.for_config, ForConfig::Stock { .. }));
        assert!(matches!(
            tactic.buy.when_config,
            WhenConfig::BelowMedian { .. }
        ));
        assert!(matches!(tactic.buy.do_config, DoConfig::Buy { .. }));
        assert!(matches!(tactic.sell.do_config, DoConfig::SellAll { .. }));
    }
}
