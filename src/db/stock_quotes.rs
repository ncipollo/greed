use crate::error::GreedError;
use crate::platform::quote::Quote;
use duckdb::{params, Connection};

pub fn insert(connection: &Connection, quote: &Quote) -> Result<(), GreedError> {
    connection.execute(
        "INSERT INTO stock_quotes (
            captured_at, quote_time, symbol, ask_price, ask_size, bid_price, bid_size
        ) VALUES (CURRENT_TIMESTAMP, ?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            quote.time,
            quote.symbol.symbol,
            quote.ask_price,
            quote.ask_size,
            quote.bid_price,
            quote.bid_size,
        ],
    )?;
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::asset::AssetSymbol;
    use crate::db::migrations;
    use crate::db::stock_quotes::insert;
    use crate::platform::quote::Quote;
    use duckdb::Connection;

    #[test]
    fn insert_adds_a_row_to_stock_quotes() {
        let connection = Connection::open_in_memory().expect("in-memory connection");
        migrations::apply(&connection).expect("migrations should apply");

        insert(&connection, &Quote::fixture(AssetSymbol::new("VTI")))
            .expect("insert should succeed");

        let count: i64 = connection
            .query_row("SELECT COUNT(*) FROM stock_quotes", [], |row| row.get(0))
            .expect("stock_quotes should be queryable");
        assert_eq!(count, 1);
    }

    #[test]
    fn insert_writes_expected_values() {
        let connection = Connection::open_in_memory().expect("in-memory connection");
        migrations::apply(&connection).expect("migrations should apply");
        let quote = Quote::fixture(AssetSymbol::new("VTI"));

        insert(&connection, &quote).expect("insert should succeed");

        let row = connection
            .query_row(
                "SELECT symbol, ask_price, ask_size, bid_price, bid_size FROM stock_quotes",
                [],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, f64>(1)?,
                        row.get::<_, u64>(2)?,
                        row.get::<_, f64>(3)?,
                        row.get::<_, u64>(4)?,
                    ))
                },
            )
            .expect("stock_quotes should be queryable");
        assert_eq!(
            row,
            (
                quote.symbol.symbol.clone(),
                quote.ask_price,
                quote.ask_size,
                quote.bid_price,
                quote.bid_size,
            )
        );
    }

    #[test]
    fn insert_returns_err_for_missing_table() {
        let connection = Connection::open_in_memory().expect("in-memory connection");

        let result = insert(&connection, &Quote::fixture(AssetSymbol::new("VTI")));

        assert!(result.is_err());
    }
}
