use crate::date::NaiveDateTimeConvert;
use crate::error::GreedError;
use crate::platform::account::Account;
use crate::platform::id::Id;
use chrono::{DateTime, NaiveDateTime, Utc};
use duckdb::{params, Connection};
use serde::Serialize;
use std::fmt::{Display, Formatter};
use std::ops::Range;

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

#[derive(Serialize)]
pub struct PortfolioStatusRecord {
    pub timestamp: DateTime<Utc>,
    #[serde(flatten)]
    pub account: Account,
}

impl Display for PortfolioStatusRecord {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} -- {}",
            self.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
            self.account
        )
    }
}

pub fn select_in_range(
    connection: &Connection,
    range: Range<DateTime<Utc>>,
) -> Result<Vec<PortfolioStatusRecord>, GreedError> {
    let mut statement = connection.prepare(
        "SELECT timestamp, buying_power, cash, equity, market_value_long, market_value_short, currency
         FROM portfolio_status
         WHERE timestamp >= ?1 AND timestamp <= ?2
         ORDER BY timestamp ASC",
    )?;
    let rows = statement.query_map(
        params![range.start.naive_utc(), range.end.naive_utc()],
        |row| {
            Ok((
                row.get::<_, NaiveDateTime>(0)?,
                row.get::<_, f64>(1)?,
                row.get::<_, f64>(2)?,
                row.get::<_, f64>(3)?,
                row.get::<_, f64>(4)?,
                row.get::<_, f64>(5)?,
                row.get::<_, String>(6)?,
            ))
        },
    )?;
    let mut records = Vec::new();
    for row in rows {
        let (
            timestamp,
            buying_power,
            cash,
            equity,
            market_value_long,
            market_value_short,
            currency,
        ) = row?;
        records.push(PortfolioStatusRecord {
            timestamp: timestamp.to_utc(),
            account: Account {
                id: Id::default(),
                buying_power,
                cash,
                currency,
                market_value_long,
                market_value_short,
                equity,
            },
        });
    }
    Ok(records)
}

#[cfg(test)]
mod test {
    use crate::db::migrations;
    use crate::db::portfolio_status::{insert, select_in_range, PortfolioStatusRecord};
    use crate::platform::account::Account;
    use chrono::{NaiveDate, TimeZone, Utc};
    use duckdb::{params, Connection};
    use serde_json::json;

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

    #[test]
    fn select_in_range_filters_and_orders_by_timestamp() {
        let connection = Connection::open_in_memory().expect("in-memory connection");
        migrations::apply(&connection).expect("migrations should apply");
        insert_fixture_row(&connection, naive_date(2026, 1, 3), 300.0);
        insert_fixture_row(&connection, naive_date(2026, 1, 1), 100.0);
        insert_fixture_row(&connection, naive_date(2026, 1, 2), 200.0);

        let start = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2026, 1, 2, 0, 0, 0).unwrap();
        let records = select_in_range(&connection, start..end).expect("select should succeed");

        let buying_powers: Vec<f64> = records.iter().map(|r| r.account.buying_power).collect();
        assert_eq!(buying_powers, vec![100.0, 200.0]);
    }

    #[test]
    fn select_in_range_returns_empty_when_nothing_in_range() {
        let connection = Connection::open_in_memory().expect("in-memory connection");
        migrations::apply(&connection).expect("migrations should apply");
        insert_fixture_row(&connection, naive_date(2026, 1, 1), 100.0);

        let start = Utc.with_ymd_and_hms(2026, 2, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2026, 2, 2, 0, 0, 0).unwrap();
        let records = select_in_range(&connection, start..end).expect("select should succeed");

        assert!(records.is_empty());
    }

    #[test]
    fn serialize_flattens_account_fields_into_the_record() {
        let record = PortfolioStatusRecord {
            timestamp: Utc.with_ymd_and_hms(2026, 1, 1, 12, 0, 0).unwrap(),
            account: Account {
                buying_power: 100.0,
                cash: 50.0,
                currency: "USD".to_string(),
                market_value_long: 10.0,
                market_value_short: 5.0,
                equity: 200.0,
                ..Default::default()
            },
        };

        let json = serde_json::to_value(&record).expect("failed to serialize");
        let expected = json!({
            "timestamp": "2026-01-01T12:00:00Z",
            "id": "00000000-0000-0000-0000-000000000000",
            "buying_power": 100.0,
            "cash": 50.0,
            "currency": "USD",
            "market_value_long": 10.0,
            "market_value_short": 5.0,
            "equity": 200.0,
        });

        assert_eq!(json, expected)
    }

    fn naive_date(year: i32, month: u32, day: u32) -> chrono::NaiveDateTime {
        NaiveDate::from_ymd_opt(year, month, day)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
    }

    fn insert_fixture_row(
        connection: &Connection,
        timestamp: chrono::NaiveDateTime,
        buying_power: f64,
    ) {
        connection
            .execute(
                "INSERT INTO portfolio_status (
                    timestamp, buying_power, cash, equity, market_value_long, market_value_short, currency
                ) VALUES (?1, ?2, 0.0, 0.0, 0.0, 0.0, 'USD')",
                params![timestamp, buying_power],
            )
            .expect("fixture insert should succeed");
    }
}
