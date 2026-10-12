use crate::db::migrations::Migration;

pub(super) const MIGRATION: Migration = Migration {
    version: 2,
    sql: "CREATE TABLE stock_quotes (
        captured_at TIMESTAMP NOT NULL,
        quote_time TIMESTAMP NOT NULL,
        symbol VARCHAR NOT NULL,
        ask_price DOUBLE NOT NULL,
        ask_size UBIGINT NOT NULL,
        bid_price DOUBLE NOT NULL,
        bid_size UBIGINT NOT NULL
    )",
};

#[cfg(test)]
mod test {
    use crate::db::migrations::apply;
    use duckdb::Connection;

    #[test]
    fn apply_creates_stock_quotes_table() {
        let connection = Connection::open_in_memory().expect("in-memory connection");

        apply(&connection).expect("apply should succeed");

        connection
            .execute(
                "INSERT INTO stock_quotes (
                    captured_at, quote_time, symbol, ask_price, ask_size, bid_price, bid_size
                ) VALUES (CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, 'VTI', 200.0, 1, 100.0, 1)",
                [],
            )
            .expect("stock_quotes should accept a row with all expected columns");
    }
}
