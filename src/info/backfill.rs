pub fn render() -> String {
    r#"GREED: BACKFILL

`greed backfill` fetches historical minute bars from the brokerage platform and
stores them in the DuckDB file, so `greed backtest` can replay them without
touching the network. Bars go in the `stock_bars` table (one row per symbol and
minute); what has been fetched is recorded in `backfill_coverage`.

USAGE
  greed backfill <SYMBOLS...> [OPTIONS]
  greed backfill --config <PATH> [OPTIONS]
  Give either symbols (e.g. `VTI SPY`) or --config, not both. With --config,
  every symbol the config trades is backfilled.

OPTIONS
  --config PATH            backfill every symbol the greed config trades
  --from D, --to D         explicit range, e.g. "2026-10-15" or "2026-10-15 14:00"
                           (UTC). --to needs --from and defaults to now; a bare
                           --to date includes that whole day; the future is
                           clamped to now.
  --days N, --months N,    range covering the last N days / months / years.
  --years N                Cannot be combined with --from.
                           With no range option the window is the last year.
  --db-path PATH           DuckDB file to write. Defaults to greed.duckdb next to
                           the config, or in the current directory when
                           backfilling symbols.
  -s, --is-simulated       use a simulated platform instead of a live account
  -p, --platform-type T    platform to fetch from when giving symbols (a config
                           names its own); conflicts with --config
  --requests-per-minute N  pace requests to N per minute. Defaults to Alpaca's
                           free-tier limit (200); raise it if your plan allows.
  --dry-run                show which windows are missing without fetching or
                           writing anything
  -v, --verbose            also show every fetched chunk

HOW IT WORKS
  Only missing data is fetched. For each symbol greed compares the requested
  range with the coverage ledger, subtracts what is already covered, and
  fetches the remaining gaps in month-sized chunks. Gaps shorter than a day are
  merged into one request. Bars and their coverage are written together, so an
  interrupted backfill never marks time as covered that has no bars.
  The most recent 15 minutes are never marked covered, because the data
  provider delays recent bars; they are fetched again next time.
  Running the same backfill again fetches little or nothing: only that newest
  stretch and any gaps are requested.

OUTPUT
  Progress (the plan, one line per month, and with -v every chunk) goes to
  STDERR. The final summary (symbols, bars inserted, requests made, elapsed
  time) goes to STDOUT, as does the --dry-run plan.

RELATED
  `greed run --record-history` records minute bars for the config's symbols
  into the same table with the same code, so a backfill only needs to cover
  older periods. `greed backtest` backfills missing data automatically before
  simulating; see `greed info backtest`.

EXAMPLES
  greed backfill VTI SPY --months 3
  greed backfill --config greed.toml --from 2026-01-01 --to 2026-06-30 --dry-run
"#
    .to_string()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn mentions_the_tables_and_the_output_split() {
        let page = render();

        assert!(page.contains("stock_bars"));
        assert!(page.contains("backfill_coverage"));
        assert!(page.contains("STDERR"));
        assert!(page.contains("STDOUT"));
    }
}
