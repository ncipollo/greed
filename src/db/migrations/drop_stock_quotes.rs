use crate::db::migrations::Migration;

/// Per-tick quote snapshots were replaced by minute bars in `stock_bars`, which the run loop
/// now records itself. This is a new migration rather than an edit to the one that created the
/// table (version 2), because existing databases have already applied that one. Any recorded
/// quote snapshots are discarded.
pub(super) const MIGRATION: Migration = Migration {
    version: 5,
    sql: "DROP TABLE IF EXISTS stock_quotes",
};

#[cfg(test)]
mod test {
    use crate::db::migrations::{
        apply, apply_with, backfill_coverage, portfolio_status, stock_bars, stock_quotes,
    };
    use duckdb::Connection;

    fn table_exists(connection: &Connection, name: &str) -> bool {
        connection
            .query_row(
                "SELECT COUNT(*) FROM information_schema.tables WHERE table_name = ?",
                [name],
                |row| row.get::<_, i64>(0),
            )
            .expect("information_schema should be queryable")
            > 0
    }

    #[test]
    fn a_fresh_database_has_no_stock_quotes_table() {
        let connection = Connection::open_in_memory().expect("in-memory connection");

        apply(&connection).expect("apply should succeed");

        assert!(!table_exists(&connection, "stock_quotes"));
        assert!(table_exists(&connection, "stock_bars"));
        assert!(table_exists(&connection, "portfolio_status"));
    }

    #[test]
    fn an_existing_database_with_recorded_quotes_has_the_table_dropped() {
        let connection = Connection::open_in_memory().expect("in-memory connection");
        // A database as the previous release left it: migrations 1 to 4 applied.
        apply_with(
            &connection,
            &[
                portfolio_status::MIGRATION,
                stock_quotes::MIGRATION,
                stock_bars::MIGRATION,
                backfill_coverage::MIGRATION,
            ],
        )
        .expect("old migrations should apply");
        connection
            .execute(
                "INSERT INTO stock_quotes (
                    captured_at, quote_time, symbol, ask_price, ask_size, bid_price, bid_size
                ) VALUES (CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, 'VTI', 200.0, 1, 100.0, 1)",
                [],
            )
            .expect("old table should accept a quote");

        apply(&connection).expect("upgrade should succeed");

        assert!(!table_exists(&connection, "stock_quotes"));
    }
}
