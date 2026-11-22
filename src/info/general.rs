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

TOPICS
  Run `greed info` for the list of topics.
"#
    .to_string()
}
