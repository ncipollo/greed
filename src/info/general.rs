pub fn render() -> String {
    r#"GREED: GENERAL USAGE

greed is an automated trading tool. It reads a config describing tactics
(buy/sell rules) and/or AI agent strategies, then repeatedly evaluates them
against a brokerage platform (currently Alpaca) on a fixed interval.

COMMANDS
  greed init <greed|strategy|agent> [PATH]   write a starter config file
  greed run <CONFIG> [-s] [--record-history] run the trading loop (blocks forever)
  greed server <CONFIG> [--port PORT] [-s]   run the loop AND serve an HTTP API
  greed status [-f]                          account status (-f adds positions/orders)
  greed orders                               recent orders
  greed quote <SYMBOLS...>                   latest quotes
  greed analyze <SYMBOLS...>                 bar analysis for symbols
  greed history status|quotes                query recorded history (DuckDB)
  greed backfill <SYMBOLS...|--config PATH>  fetch historical minute bars into DuckDB
                                             [--from D --to D | --days N --months N --years N]
                                             [--dry-run] [-v]
  greed backtest <CONFIG>                    replay a config against stored minute bars and print
                                             the final value [--starting-cash N] [--trace FILE]
                                             [--from D --to D | --days N --months N --years N] [-v]
  greed completions <shell>                  print shell completions
  greed info [TOPIC]                         this documentation

CONFIG FILES
  A config passed to `run`/`server` is TOML (see `greed info config`), or a
  CSV when the file extension is .csv (see `greed info config-simple`).
  Use `greed init` to generate a starter file for any variant.

RUN VS SERVER
  `run` only executes the trading loop. `server` runs the same loop and also
  serves an HTTP API (see `greed info server`). With --record-history, greed
  persists portfolio status and quotes to a DuckDB file (default greed.duckdb
  next to the config, override with --db-path). The file is created only when
  --record-history is set; without it no database file is written. DuckDB allows a single writer,
  so while `run`/`server` holds the write lock, a second CLI invocation would
  contend with it. The HTTP server lets other processes observe a live run
  instead.

SIMULATION
  Pass -s/--is-simulated to run/server to use a simulated platform instead of
  a live account.

BACKFILL AND BACKTEST OUTPUT
  Both commands print progress (phases, one line per month, and with -v every
  fetched chunk or simulated interval) to STDERR, and the final result to
  STDOUT, so `greed backtest cfg.toml > result.txt` captures only the result.
  Without -v, backtest hides the strategies' per-tick logging; -v keeps it.
  A backtest backfills any missing data first, then simulates with no network
  calls and no database writes. --trace accepts .json, .toml or .html.

TOPICS
  Run `greed info` for the list of topics.
"#
    .to_string()
}
