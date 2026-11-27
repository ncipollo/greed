use crate::db::migrations::Migration;

/// A ledger of every window `greed backfill` has successfully fetched. Minute
/// bars have legitimate gaps (no trades, weekends, holidays), so "is this range
/// covered?" can't be answered by counting bars; it's answered from this ledger.
pub(super) const MIGRATION: Migration = Migration {
    version: 4,
    sql: "CREATE TABLE backfill_coverage (
        symbol VARCHAR NOT NULL,
        timeframe VARCHAR NOT NULL,
        range_start TIMESTAMP NOT NULL,
        range_end TIMESTAMP NOT NULL,
        fetched_at TIMESTAMP NOT NULL
    )",
};

#[cfg(test)]
mod test {
    use crate::db::migrations::apply;
    use duckdb::Connection;

    #[test]
    fn apply_creates_backfill_coverage_table() {
        let connection = Connection::open_in_memory().expect("in-memory connection");

        apply(&connection).expect("apply should succeed");

        connection
            .execute(
                "INSERT INTO backfill_coverage (
                    symbol, timeframe, range_start, range_end, fetched_at
                ) VALUES ('VTI', '1Min', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
                [],
            )
            .expect("backfill_coverage should accept a row with all expected columns");
    }
}
