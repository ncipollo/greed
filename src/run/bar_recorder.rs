use crate::asset::AssetSymbol;
use crate::backfill::{Backfiller, RECENT_DATA_BUFFER_MINUTES};
use crate::platform::FinancialPlatform;
use chrono::{DateTime, Duration, DurationRound, Utc};
use log::{debug, warn};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// How far back each pass looks for minutes that were never recorded, e.g. because greed was
/// stopped for a while. Older gaps are left to `greed backfill`.
const CATCH_UP_DAYS: i64 = 1;

/// Keeps the local database filling in while greed runs, by recording the minute bars of the
/// symbols it watches, so backfills are only needed for older periods.
///
/// It reuses `Backfiller`, so it fetches only what the coverage ledger says is missing, and a
/// minute is only marked covered once it's complete: each pass stops short of the provider's
/// data lag (see `RECENT_DATA_BUFFER_MINUTES`) and the still-forming minute.
pub struct BarRecorder {
    backfiller: Backfiller,
    symbols: Vec<AssetSymbol>,
    last_end: Mutex<Option<DateTime<Utc>>>,
}

impl BarRecorder {
    pub fn new(
        platform: Arc<dyn FinancialPlatform>,
        db_path: PathBuf,
        symbols: Vec<AssetSymbol>,
    ) -> Self {
        Self {
            backfiller: Backfiller::new(platform, db_path),
            symbols,
            last_end: Mutex::new(None),
        }
    }

    /// Records any newly completed minutes as of `now`. Does nothing if no new minute has
    /// completed since the last call, so calling it every tick fetches at most once a minute.
    /// Failures are logged and swallowed: recording must never stop trading.
    pub async fn record(&self, now: DateTime<Utc>) {
        if self.symbols.is_empty() {
            return;
        }
        let end = completed_through(now);
        {
            let mut last_end = self.last_end.lock().unwrap();
            if last_end.is_some_and(|last| end <= last) {
                return;
            }
            // Set even if this pass fails: the next minute retries the whole window anyway.
            *last_end = Some(end);
        }
        let range = (end - Duration::days(CATCH_UP_DAYS))..end;
        match self.backfiller.backfill(&self.symbols, range).await {
            Ok(report) => debug!(
                "recorded {} new minute bars for {} symbols",
                report.total_inserted(),
                self.symbols.len()
            ),
            Err(e) => warn!("failed to record minute bars: {e}"),
        }
    }
}

/// The end of the newest minute the provider has finished with.
fn completed_through(now: DateTime<Utc>) -> DateTime<Utc> {
    let lagged = now - Duration::minutes(RECENT_DATA_BUFFER_MINUTES);
    lagged
        .duration_trunc(Duration::minutes(1))
        .unwrap_or(lagged)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::db::bootstrap_at;
    use crate::platform::bar::Bar;
    use crate::platform::bars::Bars;
    use crate::platform::MockPlatform;
    use chrono::TimeZone;
    use duckdb::Connection;
    use uuid::Uuid;

    fn at(day: u32, hour: u32, minute: u32, second: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 1, day, hour, minute, second)
            .unwrap()
    }

    fn temp_db() -> PathBuf {
        let path = std::env::temp_dir().join(format!("greed_recorder_{}.duckdb", Uuid::new_v4()));
        bootstrap_at(&path).unwrap();
        path
    }

    fn count(path: &PathBuf, table: &str) -> i64 {
        Connection::open(path)
            .unwrap()
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                row.get(0)
            })
            .unwrap()
    }

    fn bars(times: &[DateTime<Utc>]) -> Bars {
        Bars {
            symbol: AssetSymbol::new("VTI"),
            bars: times
                .iter()
                .map(|t| Bar {
                    timestamp: *t,
                    open: 1.0,
                    high: 2.0,
                    low: 0.5,
                    close: 1.5,
                    volume: 10,
                })
                .collect(),
        }
    }

    fn recorder(platform: Arc<MockPlatform>, path: &std::path::Path) -> BarRecorder {
        BarRecorder::new(platform, path.to_path_buf(), vec![AssetSymbol::new("VTI")])
    }

    #[test]
    fn completed_through_lags_the_provider_and_drops_the_forming_minute() {
        let end = completed_through(at(5, 14, 45, 37));

        assert_eq!(end, at(5, 14, 30, 0));
    }

    #[tokio::test]
    async fn records_bars_and_coverage_for_completed_minutes() {
        let path = temp_db();
        let platform =
            Arc::new(MockPlatform::new().with_bars(bars(&[at(5, 14, 28, 0), at(5, 14, 29, 0)])));
        let recorder = recorder(platform.clone(), &path);

        recorder.record(at(5, 14, 45, 10)).await;

        assert_eq!(count(&path, "stock_bars"), 2);
        assert_eq!(count(&path, "backfill_coverage"), 1);
        let requests = platform.bar_requests();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].end, at(5, 14, 30, 0));
        std::fs::remove_file(&path).ok();
    }

    #[tokio::test]
    async fn fetches_at_most_once_per_minute() {
        let path = temp_db();
        let platform = Arc::new(MockPlatform::new());
        let recorder = recorder(platform.clone(), &path);

        recorder.record(at(5, 14, 45, 5)).await;
        recorder.record(at(5, 14, 45, 20)).await;
        recorder.record(at(5, 14, 45, 55)).await;
        let after_one_minute = platform.bar_requests().len();
        recorder.record(at(5, 14, 46, 5)).await;

        assert_eq!(after_one_minute, 1);
        assert_eq!(platform.bar_requests().len(), 2);
        std::fs::remove_file(&path).ok();
    }

    #[tokio::test]
    async fn after_downtime_only_the_missing_minutes_are_fetched() {
        let path = temp_db();
        let platform = Arc::new(MockPlatform::new());
        let first = recorder(platform.clone(), &path);
        first.record(at(5, 14, 45, 0)).await;

        // A new recorder (a restarted run) sees the ledger and fetches only the new stretch.
        let second = recorder(platform.clone(), &path);
        second.record(at(5, 14, 50, 0)).await;

        let requests = platform.bar_requests();
        let last = requests.last().unwrap();
        assert_eq!(last.start, at(5, 14, 30, 0));
        assert_eq!(last.end, at(5, 14, 35, 0));
        std::fs::remove_file(&path).ok();
    }

    #[tokio::test]
    async fn a_failed_fetch_is_swallowed_and_retried_next_minute() {
        let path = temp_db();
        let platform = Arc::new(
            MockPlatform::new().with_bars_results(vec![Err("api down".to_string()), Ok(bars(&[]))]),
        );
        let recorder = recorder(platform.clone(), &path);

        recorder.record(at(5, 14, 45, 0)).await;
        assert_eq!(count(&path, "backfill_coverage"), 0);
        recorder.record(at(5, 14, 46, 0)).await;

        assert_eq!(platform.bar_requests().len(), 2);
        assert_eq!(count(&path, "backfill_coverage"), 1);
        std::fs::remove_file(&path).ok();
    }

    #[tokio::test]
    async fn without_symbols_nothing_is_fetched() {
        let path = temp_db();
        let platform = Arc::new(MockPlatform::new());
        let recorder = BarRecorder::new(platform.clone(), path.clone(), Vec::new());

        recorder.record(at(5, 14, 45, 0)).await;

        assert!(platform.bar_requests().is_empty());
        std::fs::remove_file(&path).ok();
    }
}
