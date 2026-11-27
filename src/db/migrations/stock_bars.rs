use crate::db::migrations::Migration;

/// Historical bars fetched by `greed backfill`. Kept apart from `stock_quotes`,
/// which holds live snapshots recorded by `run --record-history`, so backfilled
/// history is never mistaken for real recorded activity. The primary key makes
/// re-inserting an overlapping window a no-op.
pub(super) const MIGRATION: Migration = Migration {
    version: 3,
    sql: "CREATE TABLE stock_bars (
        symbol VARCHAR NOT NULL,
        timeframe VARCHAR NOT NULL,
        bar_time TIMESTAMP NOT NULL,
        open DOUBLE NOT NULL,
        high DOUBLE NOT NULL,
        low DOUBLE NOT NULL,
        close DOUBLE NOT NULL,
        volume UBIGINT NOT NULL,
        fetched_at TIMESTAMP NOT NULL,
        PRIMARY KEY (symbol, timeframe, bar_time)
    )",
};

#[cfg(test)]
mod test {
    use crate::db::migrations::apply;
    use duckdb::Connection;

    #[test]
    fn apply_creates_stock_bars_table() {
        let connection = Connection::open_in_memory().expect("in-memory connection");

        apply(&connection).expect("apply should succeed");

        connection
            .execute(
                "INSERT INTO stock_bars (
                    symbol, timeframe, bar_time, open, high, low, close, volume, fetched_at
                ) VALUES ('VTI', '1Min', CURRENT_TIMESTAMP, 1.0, 2.0, 0.5, 1.5, 10, CURRENT_TIMESTAMP)",
                [],
            )
            .expect("stock_bars should accept a row with all expected columns");
    }

    #[test]
    fn stock_bars_rejects_a_duplicate_bar() {
        let connection = Connection::open_in_memory().expect("in-memory connection");
        apply(&connection).expect("apply should succeed");
        let insert = "INSERT INTO stock_bars (
            symbol, timeframe, bar_time, open, high, low, close, volume, fetched_at
        ) VALUES ('VTI', '1Min', TIMESTAMP '2026-01-01 10:00:00', 1.0, 2.0, 0.5, 1.5, 10, CURRENT_TIMESTAMP)";
        connection.execute(insert, []).expect("first insert");

        assert!(connection.execute(insert, []).is_err());
    }
}
