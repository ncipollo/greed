use crate::error::GreedError;
use crate::platform::account::Account;
use duckdb::{params, Connection};

pub fn insert(connection: &Connection, account: &Account) -> Result<(), GreedError> {
    connection.execute(
        "INSERT INTO portfolio_status (
            timestamp, buying_power, cash, equity, market_value_long, market_value_short, currency
        ) VALUES (CURRENT_TIMESTAMP, ?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            account.buying_power,
            account.cash,
            account.equity,
            account.market_value_long,
            account.market_value_short,
            account.currency,
        ],
    )?;
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::db::migrations;
    use crate::db::portfolio_status::insert;
    use crate::platform::account::Account;
    use duckdb::Connection;

    #[test]
    fn insert_adds_a_row_to_portfolio_status() {
        let connection = Connection::open_in_memory().expect("in-memory connection");
        migrations::apply(&connection).expect("migrations should apply");

        insert(&connection, &Account::fixture()).expect("insert should succeed");

        let count: i64 = connection
            .query_row("SELECT COUNT(*) FROM portfolio_status", [], |row| {
                row.get(0)
            })
            .expect("portfolio_status should be queryable");
        assert_eq!(count, 1);
    }

    #[test]
    fn insert_writes_expected_values() {
        let connection = Connection::open_in_memory().expect("in-memory connection");
        migrations::apply(&connection).expect("migrations should apply");
        let account = Account::fixture();

        insert(&connection, &account).expect("insert should succeed");

        let row = connection
            .query_row(
                "SELECT buying_power, cash, equity, market_value_long, market_value_short, currency
                 FROM portfolio_status",
                [],
                |row| {
                    Ok((
                        row.get::<_, f64>(0)?,
                        row.get::<_, f64>(1)?,
                        row.get::<_, f64>(2)?,
                        row.get::<_, f64>(3)?,
                        row.get::<_, f64>(4)?,
                        row.get::<_, String>(5)?,
                    ))
                },
            )
            .expect("portfolio_status should be queryable");
        assert_eq!(
            row,
            (
                account.buying_power,
                account.cash,
                account.equity,
                account.market_value_long,
                account.market_value_short,
                account.currency,
            )
        );
    }

    #[test]
    fn insert_returns_err_for_missing_table() {
        let connection = Connection::open_in_memory().expect("in-memory connection");

        let result = insert(&connection, &Account::fixture());

        assert!(result.is_err());
    }
}
