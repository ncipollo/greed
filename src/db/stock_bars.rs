use crate::asset::AssetSymbol;
use crate::db::backfill_coverage;
use crate::error::GreedError;
use crate::platform::bar::time_frame::TimeFrame;
use crate::platform::bar::Bar;
use chrono::{DateTime, Utc};
use duckdb::{params, Connection};
use std::ops::Range;

/// Stores one fetched `window` of bars and records it in `backfill_coverage`,
/// atomically: either both land or neither does, so an interrupted backfill
/// never claims coverage for bars it didn't save.
///
/// Bars already stored (same symbol, timeframe and time) are skipped, so
/// overlapping windows don't create duplicates. Returns how many bars were new.
pub fn insert_chunk(
    connection: &mut Connection,
    symbol: &AssetSymbol,
    timeframe: &TimeFrame,
    window: &Range<DateTime<Utc>>,
    bars: &[Bar],
) -> Result<usize, GreedError> {
    let tx = connection.transaction()?;
    let mut inserted = 0;
    {
        let mut statement = tx.prepare(
            "INSERT INTO stock_bars (
                symbol, timeframe, bar_time, open, high, low, close, volume, fetched_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, CURRENT_TIMESTAMP)
            ON CONFLICT DO NOTHING",
        )?;
        for bar in bars {
            inserted += statement.execute(params![
                symbol.symbol,
                timeframe.as_str(),
                bar.timestamp.naive_utc(),
                bar.open,
                bar.high,
                bar.low,
                bar.close,
                bar.volume as u64,
            ])?;
        }
    }
    backfill_coverage::insert(&tx, symbol, timeframe, window)?;
    tx.commit()?;
    Ok(inserted)
}

/// Stored bars for `symbol` at `timeframe` with `range.start <= bar_time <
/// range.end`, oldest first.
pub fn load(
    connection: &Connection,
    symbol: &AssetSymbol,
    timeframe: &TimeFrame,
    range: &Range<DateTime<Utc>>,
) -> Result<Vec<Bar>, GreedError> {
    let mut statement = connection.prepare(
        "SELECT bar_time, open, high, low, close, volume FROM stock_bars
         WHERE symbol = ?1 AND timeframe = ?2 AND bar_time >= ?3 AND bar_time < ?4
         ORDER BY bar_time",
    )?;
    let rows = statement.query_map(
        params![
            symbol.symbol,
            timeframe.as_str(),
            range.start.naive_utc(),
            range.end.naive_utc()
        ],
        |row| {
            Ok(Bar {
                timestamp: row.get::<_, chrono::NaiveDateTime>(0)?.and_utc(),
                open: row.get(1)?,
                high: row.get(2)?,
                low: row.get(3)?,
                close: row.get(4)?,
                volume: row.get::<_, u64>(5)? as usize,
            })
        },
    )?;
    let mut bars = Vec::new();
    for row in rows {
        bars.push(row?);
    }
    Ok(bars)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::db::migrations;
    use chrono::TimeZone;

    fn at(day: u32, hour: u32, minute: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 1, day, hour, minute, 0).unwrap()
    }

    fn window() -> Range<DateTime<Utc>> {
        at(1, 0, 0)..at(2, 0, 0)
    }

    fn bar(timestamp: DateTime<Utc>, close: f64) -> Bar {
        Bar {
            timestamp,
            open: close - 1.0,
            high: close + 1.0,
            low: close - 2.0,
            close,
            volume: 100,
        }
    }

    fn migrated() -> Connection {
        let connection = Connection::open_in_memory().expect("in-memory connection");
        migrations::apply(&connection).expect("migrations should apply");
        connection
    }

    fn count(connection: &Connection, table: &str) -> i64 {
        connection
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                row.get(0)
            })
            .expect("table should be queryable")
    }

    fn insert_vti(connection: &mut Connection, bars: &[Bar]) -> Result<usize, GreedError> {
        insert_chunk(
            connection,
            &AssetSymbol::new("VTI"),
            &TimeFrame::OneMinute,
            &window(),
            bars,
        )
    }

    #[test]
    fn insert_chunk_writes_a_row_per_bar_and_a_coverage_row() {
        let mut connection = migrated();
        let bars = vec![bar(at(1, 14, 30), 100.0), bar(at(1, 14, 31), 101.0)];

        let inserted = insert_vti(&mut connection, &bars).expect("insert should succeed");

        assert_eq!(inserted, 2);
        assert_eq!(count(&connection, "stock_bars"), 2);
        assert_eq!(count(&connection, "backfill_coverage"), 1);
    }

    #[test]
    fn insert_chunk_writes_expected_values() {
        let mut connection = migrated();

        insert_vti(&mut connection, &[bar(at(1, 14, 30), 100.0)]).expect("insert");

        let row = connection
            .query_row(
                "SELECT symbol, timeframe, open, high, low, close, volume FROM stock_bars",
                [],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, f64>(2)?,
                        row.get::<_, f64>(3)?,
                        row.get::<_, f64>(4)?,
                        row.get::<_, f64>(5)?,
                        row.get::<_, u64>(6)?,
                    ))
                },
            )
            .expect("stock_bars should be queryable");
        assert_eq!(
            row,
            (
                "VTI".to_string(),
                "1Min".to_string(),
                99.0,
                101.0,
                98.0,
                100.0,
                100
            )
        );
    }

    #[test]
    fn insert_chunk_skips_bars_that_are_already_stored() {
        let mut connection = migrated();
        let first = vec![bar(at(1, 14, 30), 100.0), bar(at(1, 14, 31), 101.0)];
        let overlapping = vec![bar(at(1, 14, 31), 101.0), bar(at(1, 14, 32), 102.0)];
        insert_vti(&mut connection, &first).expect("first insert");

        let inserted = insert_vti(&mut connection, &overlapping).expect("second insert");

        assert_eq!(inserted, 1);
        assert_eq!(count(&connection, "stock_bars"), 3);
    }

    #[test]
    fn insert_chunk_with_no_bars_still_records_coverage() {
        let mut connection = migrated();

        let inserted = insert_vti(&mut connection, &[]).expect("insert should succeed");

        assert_eq!(inserted, 0);
        assert_eq!(count(&connection, "stock_bars"), 0);
        assert_eq!(count(&connection, "backfill_coverage"), 1);
    }

    #[test]
    fn insert_chunk_keeps_symbols_and_timeframes_separate() {
        let mut connection = migrated();
        let same_bar = [bar(at(1, 14, 30), 100.0)];
        insert_vti(&mut connection, &same_bar).expect("vti");

        insert_chunk(
            &mut connection,
            &AssetSymbol::new("SPY"),
            &TimeFrame::OneMinute,
            &window(),
            &same_bar,
        )
        .expect("spy");
        insert_chunk(
            &mut connection,
            &AssetSymbol::new("VTI"),
            &TimeFrame::OneHour,
            &window(),
            &same_bar,
        )
        .expect("vti hourly");

        assert_eq!(count(&connection, "stock_bars"), 3);
    }

    #[test]
    fn insert_chunk_returns_err_for_missing_table_and_writes_nothing() {
        let mut connection = Connection::open_in_memory().expect("in-memory connection");

        let result = insert_vti(&mut connection, &[bar(at(1, 14, 30), 100.0)]);

        assert!(result.is_err());
    }

    #[test]
    fn insert_chunk_rolls_back_bars_when_coverage_cannot_be_recorded() {
        let mut connection = migrated();
        connection
            .execute_batch("DROP TABLE backfill_coverage")
            .expect("drop coverage table");

        let result = insert_vti(&mut connection, &[bar(at(1, 14, 30), 100.0)]);

        assert!(result.is_err());
        assert_eq!(count(&connection, "stock_bars"), 0);
    }
}
