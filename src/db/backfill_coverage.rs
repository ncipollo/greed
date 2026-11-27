use crate::asset::AssetSymbol;
use crate::error::GreedError;
use crate::platform::bar::time_frame::TimeFrame;
use chrono::{DateTime, Utc};
use duckdb::{params, Connection};
use std::ops::Range;

/// Records that `window` was fetched in full for `symbol` at `timeframe`. An
/// empty fetch (a holiday, an illiquid minute range) is still coverage.
pub fn insert(
    connection: &Connection,
    symbol: &AssetSymbol,
    timeframe: &TimeFrame,
    window: &Range<DateTime<Utc>>,
) -> Result<(), GreedError> {
    connection.execute(
        "INSERT INTO backfill_coverage (
            symbol, timeframe, range_start, range_end, fetched_at
        ) VALUES (?1, ?2, ?3, ?4, CURRENT_TIMESTAMP)",
        params![
            symbol.symbol,
            timeframe.as_str(),
            window.start.naive_utc(),
            window.end.naive_utc(),
        ],
    )?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::db::migrations;
    use chrono::{NaiveDateTime, TimeZone};

    #[test]
    fn insert_records_the_window() {
        let connection = Connection::open_in_memory().expect("in-memory connection");
        migrations::apply(&connection).expect("migrations should apply");
        let window = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap()
            ..Utc.with_ymd_and_hms(2026, 2, 1, 0, 0, 0).unwrap();

        insert(
            &connection,
            &AssetSymbol::new("VTI"),
            &TimeFrame::OneMinute,
            &window,
        )
        .expect("insert should succeed");

        let row = connection
            .query_row(
                "SELECT symbol, timeframe, range_start, range_end FROM backfill_coverage",
                [],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, NaiveDateTime>(2)?,
                        row.get::<_, NaiveDateTime>(3)?,
                    ))
                },
            )
            .expect("coverage should be queryable");
        assert_eq!(
            row,
            (
                "VTI".to_string(),
                "1Min".to_string(),
                window.start.naive_utc(),
                window.end.naive_utc()
            )
        );
    }

    #[test]
    fn insert_returns_err_for_missing_table() {
        let connection = Connection::open_in_memory().expect("in-memory connection");
        let window = Utc::now()..Utc::now();

        let result = insert(
            &connection,
            &AssetSymbol::new("VTI"),
            &TimeFrame::OneMinute,
            &window,
        );

        assert!(result.is_err());
    }
}
