use crate::db::migrations::Migration;

pub(super) const MIGRATION: Migration = Migration {
    version: 1,
    sql: "CREATE TABLE portfolio_status (
        timestamp TIMESTAMP NOT NULL,
        buying_power DOUBLE NOT NULL,
        cash DOUBLE NOT NULL,
        equity DOUBLE NOT NULL,
        market_value_long DOUBLE NOT NULL,
        market_value_short DOUBLE NOT NULL,
        currency VARCHAR NOT NULL
    )",
};

#[cfg(test)]
mod test {
    use crate::db::migrations::apply;
    use duckdb::Connection;

    #[test]
    fn apply_creates_portfolio_status_table() {
        let connection = Connection::open_in_memory().expect("in-memory connection");

        apply(&connection).expect("apply should succeed");

        connection
            .execute(
                "INSERT INTO portfolio_status (
                    timestamp, buying_power, cash, equity, market_value_long, market_value_short, currency
                ) VALUES (CURRENT_TIMESTAMP, 1000.0, 500.0, 500.0, 0.0, 0.0, 'USD')",
                [],
            )
            .expect("portfolio_status should accept a row with all expected columns");
    }
}
