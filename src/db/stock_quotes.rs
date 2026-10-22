use crate::asset::AssetSymbol;
use crate::date::NaiveDateTimeConvert;
use crate::error::GreedError;
use crate::platform::quote::Quote;
use chrono::{DateTime, NaiveDateTime, Utc};
use duckdb::{params, Connection};
use std::ops::Range;

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

pub fn insert_many(connection: &mut Connection, quotes: &[Quote]) -> Result<(), GreedError> {
    if quotes.is_empty() {
        return Ok(());
    }
    let tx = connection.transaction()?;
    for quote in quotes {
        insert(&tx, quote)?;
    }
    tx.commit()?;
    Ok(())
}

pub fn select_in_range(
    connection: &Connection,
    range: Range<DateTime<Utc>>,
) -> Result<Vec<Quote>, GreedError> {
    let mut statement = connection.prepare(
        "SELECT quote_time, symbol, ask_price, ask_size, bid_price, bid_size
         FROM stock_quotes
         WHERE quote_time >= ?1 AND quote_time <= ?2
         ORDER BY quote_time ASC",
    )?;
    let rows = statement.query_map(
        params![range.start.naive_utc(), range.end.naive_utc()],
        |row| {
            Ok((
                row.get::<_, NaiveDateTime>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, f64>(2)?,
                row.get::<_, u64>(3)?,
                row.get::<_, f64>(4)?,
                row.get::<_, u64>(5)?,
            ))
        },
    )?;
    let mut quotes = Vec::new();
    for row in rows {
        let (quote_time, symbol, ask_price, ask_size, bid_price, bid_size) = row?;
        quotes.push(Quote {
            time: quote_time.to_utc(),
            ask_price,
            ask_size,
            bid_price,
            bid_size,
            symbol: AssetSymbol::from(symbol),
        });
    }
    Ok(quotes)
}

#[cfg(test)]
mod test {
    use crate::asset::AssetSymbol;
    use crate::date::NaiveDateTimeConvert;
    use crate::db::migrations;
    use crate::db::stock_quotes::{insert, insert_many, select_in_range};
    use crate::platform::quote::Quote;
    use chrono::{NaiveDate, TimeZone, Utc};
    use duckdb::{params, Connection};

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

    #[test]
    fn insert_many_writes_a_row_per_quote() {
        let mut connection = Connection::open_in_memory().expect("in-memory connection");
        migrations::apply(&connection).expect("migrations should apply");
        let quotes = vec![
            Quote::fixture(AssetSymbol::new("VTI")),
            Quote::fixture(AssetSymbol::new("UVXY")),
        ];

        insert_many(&mut connection, &quotes).expect("insert_many should succeed");

        let count: i64 = connection
            .query_row("SELECT COUNT(*) FROM stock_quotes", [], |row| row.get(0))
            .expect("stock_quotes should be queryable");
        assert_eq!(count, 2);
    }

    #[test]
    fn insert_many_is_noop_for_empty_slice() {
        let mut connection = Connection::open_in_memory().expect("in-memory connection");
        migrations::apply(&connection).expect("migrations should apply");

        insert_many(&mut connection, &[]).expect("insert_many should succeed");

        let count: i64 = connection
            .query_row("SELECT COUNT(*) FROM stock_quotes", [], |row| row.get(0))
            .expect("stock_quotes should be queryable");
        assert_eq!(count, 0);
    }

    #[test]
    fn insert_many_returns_err_for_missing_table() {
        let mut connection = Connection::open_in_memory().expect("in-memory connection");

        let result = insert_many(&mut connection, &[Quote::fixture(AssetSymbol::new("VTI"))]);

        assert!(result.is_err());
    }

    #[test]
    fn select_in_range_filters_and_orders_by_quote_time() {
        let connection = Connection::open_in_memory().expect("in-memory connection");
        migrations::apply(&connection).expect("migrations should apply");
        insert_fixture_row(&connection, naive_date(2026, 1, 3), "VTI", 300.0);
        insert_fixture_row(&connection, naive_date(2026, 1, 1), "VTI", 100.0);
        insert_fixture_row(&connection, naive_date(2026, 1, 2), "VTI", 200.0);

        let start = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2026, 1, 2, 0, 0, 0).unwrap();
        let quotes = select_in_range(&connection, start..end).expect("select should succeed");

        let ask_prices: Vec<f64> = quotes.iter().map(|q| q.ask_price).collect();
        assert_eq!(ask_prices, vec![100.0, 200.0]);
    }

    #[test]
    fn select_in_range_returns_empty_when_nothing_in_range() {
        let connection = Connection::open_in_memory().expect("in-memory connection");
        migrations::apply(&connection).expect("migrations should apply");
        insert_fixture_row(&connection, naive_date(2026, 1, 1), "VTI", 100.0);

        let start = Utc.with_ymd_and_hms(2026, 2, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2026, 2, 2, 0, 0, 0).unwrap();
        let quotes = select_in_range(&connection, start..end).expect("select should succeed");

        assert!(quotes.is_empty());
    }

    #[test]
    fn select_in_range_maps_fields_into_quote() {
        let connection = Connection::open_in_memory().expect("in-memory connection");
        migrations::apply(&connection).expect("migrations should apply");
        let quote_time = naive_date(2026, 1, 1);
        connection
            .execute(
                "INSERT INTO stock_quotes (
                    captured_at, quote_time, symbol, ask_price, ask_size, bid_price, bid_size
                ) VALUES (CURRENT_TIMESTAMP, ?1, 'AAPL', 200.0, 5, 100.0, 7)",
                params![quote_time],
            )
            .expect("fixture insert should succeed");

        let start = Utc.with_ymd_and_hms(2025, 12, 31, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2026, 1, 2, 0, 0, 0).unwrap();
        let quotes = select_in_range(&connection, start..end).expect("select should succeed");

        assert_eq!(
            quotes,
            vec![Quote {
                time: quote_time.to_utc(),
                ask_price: 200.0,
                ask_size: 5,
                bid_price: 100.0,
                bid_size: 7,
                symbol: AssetSymbol::new("AAPL"),
            }]
        );
    }

    fn naive_date(year: i32, month: u32, day: u32) -> chrono::NaiveDateTime {
        NaiveDate::from_ymd_opt(year, month, day)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
    }

    fn insert_fixture_row(
        connection: &Connection,
        quote_time: chrono::NaiveDateTime,
        symbol: &str,
        ask_price: f64,
    ) {
        connection
            .execute(
                "INSERT INTO stock_quotes (
                    captured_at, quote_time, symbol, ask_price, ask_size, bid_price, bid_size
                ) VALUES (CURRENT_TIMESTAMP, ?1, ?2, ?3, 0, 0.0, 0)",
                params![quote_time, symbol, ask_price],
            )
            .expect("fixture insert should succeed");
    }
}
