pub fn render() -> String {
    r#"GREED: BACKTEST

`greed backtest <CONFIG>` replays a greed config against stored historical
minute bars and prints how it would have done. It trades a simulated account;
nothing touches a live account, and the simulation itself makes no network
calls and no database writes.

USAGE
  greed backtest <CONFIG> [OPTIONS]
  CONFIG is any config that `greed run` accepts (see `greed info config`).
  Agent strategies cannot be replayed (they call an LLM and the web), so a
  config containing one is rejected.

OPTIONS
  --starting-cash N        starting buying power of the simulated account
                           (default 100000)
  --from D, --to D         explicit range, e.g. "2026-10-15" or "2026-10-15 14:00"
                           (UTC). --to needs --from and defaults to now; a bare
                           --to date includes that whole day.
  --days N, --months N,    range covering the last N days / months / years.
  --years N                Cannot be combined with --from.
                           With no range option the window is the last year.
  --db-path PATH           DuckDB file holding the bars. Defaults to greed.duckdb
                           next to the config.
  --trace PATH             write a trace of the run. The format comes from the
                           extension: .json, .toml, or .html (a self-contained
                           page you can open offline).
  --fee DOLLARS            flat fee charged on every fill (default 0)
  --slippage-bps BPS       move every fill price against you by this many basis
                           points: up for buys, down for sells; 100 bps = 1%
                           (default 0)
  -v, --verbose            also show every simulated interval and the
                           strategies' own per-tick logging

HOW IT RUNS
  1. Preflight: the stored bars are checked against the range. Anything missing
     is backfilled first from the platform named in the config (the same
     gap-only fetch as `greed backfill`; see `greed info backfill`). If the
     data is already stored, no platform is created and no network is used.
  2. Simulation: the bars are loaded into memory and a simulated clock steps
     through the range at the config's interval, running every strategy once
     per step.

SIMULATION MODEL
  The simulated clock steps through the range at the config's interval.
  Strategies only ever see bars at or before the clock, never later ones.
  Orders are market orders. An order placed at one step waits and fills at the
  open of the first bar after the decision, never at the price the strategy
  just saw; cash and holdings are checked when it fills, not when it is placed.
  Limit, stop and bracket orders are rejected as unsupported.
  Fees and slippage default to 0, a free and frictionless market.
  Stretches with no bar for 15 minutes count as the market being closed and are
  skipped. Orders still waiting when the range ends are reported as unfilled.
  Holdings are valued at the last known close, however old.

OUTPUT
  Progress (phases, one line per month, and with -v every interval) goes to
  STDERR. The result goes to STDOUT, so `greed backtest cfg.toml > result.txt`
  captures only the result: the range, starting cash, trade count, the
  assumptions used (fill rule, fee, slippage), any unfilled orders, the return
  and the final value. Without -v the strategies' per-tick logging is hidden.

TRACE
  --trace records every trade and rejected order (time, strategy, tactic,
  symbol, side, quantity, price, fee, slippage, cash and value afterwards), a
  portfolio-value point per simulated day, and the assumptions the run used
  (fill rule, order types, fee, slippage, starting cash, interval, closed-market
  rule and each order that never filled). The .html page adds a chart and
  filters for browsing it.

EXAMPLES
  greed backtest greed.toml --months 6
  greed backtest greed.toml --from 2026-01-01 --fee 1 --slippage-bps 5 --trace run.html
"#
    .to_string()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn states_the_fill_rule_and_the_output_split() {
        let page = render();

        assert!(page.contains("open of the first bar after the decision"));
        assert!(page.contains("STDERR"));
        assert!(page.contains("STDOUT"));
    }
}
