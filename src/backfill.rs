mod chunks;
mod coverage;
mod report;
mod target;

pub use coverage::{BackfillPlan, CoverageChecker};
pub use report::{BackfillReport, SymbolReport};
pub use target::BackfillTarget;

use crate::asset::AssetSymbol;
use crate::backfill::chunks::month_chunks;
use crate::backfill::target::target_from_config;
use crate::config::platform::PlatformType;
use crate::db;
use crate::db::stock_bars;
use crate::error::GreedError;
use crate::interval_set;
use crate::platform;
use crate::platform::args::PlatformArgs;
use crate::platform::bar::bar_request::BarRequest;
use crate::platform::bar::time_frame::TimeFrame;
use crate::platform::FinancialPlatform;
use crate::time_range::TimeRangeSpec;
use chrono::{DateTime, Duration, Utc};
use itertools::Itertools;
use log::info;
use std::ops::Range;
use std::path::PathBuf;
use std::sync::Arc;

/// The newest data we ask for. Providers lag slightly on recent bars (Alpaca's
/// free plan withholds the last 15 minutes of some feeds), and a half-formed
/// minute shouldn't be recorded as fully covered. The tail is picked up by a
/// later backfill.
const RECENT_DATA_BUFFER_MINUTES: i64 = 15;

/// Gaps separated by less than this many days are fetched as one request:
/// refetching a little covered data is cheaper than many tiny requests.
const COALESCE_GAP_DAYS: i64 = 1;

/// Everything `greed backfill` needs, as parsed from the command line.
#[derive(Debug)]
pub struct BackfillRequest {
    pub target: BackfillTarget,
    pub time_range: TimeRangeSpec,
    /// Overrides the DuckDB path. Defaults next to the config, or `greed.duckdb`
    /// in the working directory when backfilling explicit symbols.
    pub db_path: Option<PathBuf>,
    /// Used for explicit symbols; a config names its own platform.
    pub platform_type: PlatformType,
    pub platform_args: PlatformArgs,
    /// Report what would be fetched without any network calls or database writes.
    pub dry_run: bool,
}

/// What `greed backfill` did: fetched data, or (dry run) only planned.
#[derive(Debug)]
pub enum BackfillOutcome {
    Fetched(BackfillReport),
    Planned(BackfillPlan),
}

impl std::fmt::Display for BackfillOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fetched(report) => report.fmt(f),
            Self::Planned(plan) => plan.fmt(f),
        }
    }
}

/// Fetches minute bars for the target over the requested range and saves them
/// to the local DuckDB database.
pub async fn backfill(request: BackfillRequest) -> Result<BackfillOutcome, GreedError> {
    let range = available_range(request.time_range.resolve(Utc::now())?, Utc::now())?;
    let (symbols, platform_type, db_path) = match request.target {
        BackfillTarget::Symbols(symbols) => (
            symbols,
            request.platform_type,
            db::path_for_override(request.db_path.as_deref()),
        ),
        BackfillTarget::Config(config_path) => {
            let target = target_from_config(&config_path).await?;
            let db_path = db::path_for_run(&config_path, request.db_path.as_deref());
            (target.symbols, target.platform, db_path)
        }
    };
    if symbols.is_empty() {
        return Err(GreedError::new("no symbols to backfill"));
    }
    if request.dry_run {
        let plan = CoverageChecker::plan(&db_path, &symbols, &TimeFrame::OneMinute, range).await?;
        return Ok(BackfillOutcome::Planned(plan));
    }
    let platform = platform::for_type(&platform_type, request.platform_args)?;
    // Create the file and schema up front; the connection is dropped right away
    // so the write lock is only ever held for each chunk's brief insert.
    db::bootstrap_at_with_retry(&db_path).await?;
    let report = Backfiller::new(platform, db_path)
        .backfill(&symbols, range)
        .await?;
    Ok(BackfillOutcome::Fetched(report))
}

/// Pulls bars for symbols a window at a time, saving each window as soon as it
/// arrives so an interrupted run keeps everything fetched so far.
pub struct Backfiller {
    platform: Arc<dyn FinancialPlatform>,
    db_path: PathBuf,
}

impl Backfiller {
    pub fn new(platform: Arc<dyn FinancialPlatform>, db_path: PathBuf) -> Self {
        Self { platform, db_path }
    }

    /// The schema must already exist (see `db::bootstrap_at`). Request pacing
    /// is the platform's job (see `FinancialPlatform::rate_limit`).
    pub async fn backfill(
        &self,
        symbols: &[AssetSymbol],
        range: Range<DateTime<Utc>>,
    ) -> Result<BackfillReport, GreedError> {
        let mut reports = Vec::new();
        for symbol in symbols.iter().unique() {
            let mut report = SymbolReport::new(symbol.clone());
            let gaps = self.gaps_for(symbol, &range).await?;
            if gaps.is_empty() {
                info!("⏭️  {} already complete, skipping", symbol.symbol);
                report.already_complete = true;
            }
            let windows = gaps.iter().flat_map(month_chunks).collect_vec();
            for window in &windows {
                let (fetched, inserted) = self.backfill_window(symbol, window).await?;
                report.record_window(fetched, inserted);
            }
            reports.push(report);
        }
        Ok(BackfillReport {
            range,
            symbols: reports,
        })
    }

    /// What the ledger says is missing for `symbol`, with nearby gaps joined
    /// into fewer, larger requests. The read connection is dropped before any
    /// network call so it never holds a lock while fetching.
    async fn gaps_for(
        &self,
        symbol: &AssetSymbol,
        range: &Range<DateTime<Utc>>,
    ) -> Result<Vec<Range<DateTime<Utc>>>, GreedError> {
        let plan = CoverageChecker::plan(
            &self.db_path,
            std::slice::from_ref(symbol),
            &TimeFrame::OneMinute,
            range.clone(),
        )
        .await?;
        Ok(interval_set::coalesce(
            plan.gaps_for(symbol).to_vec(),
            Duration::days(COALESCE_GAP_DAYS),
        ))
    }

    async fn backfill_window(
        &self,
        symbol: &AssetSymbol,
        window: &Range<DateTime<Utc>>,
    ) -> Result<(usize, usize), GreedError> {
        let timeframe = TimeFrame::OneMinute;
        let bars = self
            .platform
            .bars(BarRequest {
                symbol: symbol.clone(),
                limit: None,
                start: window.start,
                end: window.end,
                timeframe: timeframe.clone(),
            })
            .await
            .map_err(|e| window_error(symbol, window, &e))?;
        // The network call is done, so the write lock is held only for this insert.
        let mut connection = db::open_for_write_with_retry(&self.db_path).await?;
        let inserted =
            stock_bars::insert_chunk(&mut connection, symbol, &timeframe, window, &bars.bars)
                .map_err(|e| window_error(symbol, window, &e))?;
        info!(
            "📥 {} {} to {}: {} bars fetched, {} new",
            symbol.symbol,
            window.start.format("%Y-%m-%d"),
            window.end.format("%Y-%m-%d"),
            bars.bars.len(),
            inserted
        );
        Ok((bars.bars.len(), inserted))
    }
}

fn window_error(
    symbol: &AssetSymbol,
    window: &Range<DateTime<Utc>>,
    cause: &GreedError,
) -> GreedError {
    GreedError::new(&format!(
        "backfill of {} for {} to {} failed: {cause}",
        symbol.symbol, window.start, window.end
    ))
}

/// Trims the end of `range` back from `now` by the recent-data buffer. Errors
/// when nothing is left, e.g. `--days 0`-sized windows or a range that starts
/// inside the buffer.
fn available_range(
    range: Range<DateTime<Utc>>,
    now: DateTime<Utc>,
) -> Result<Range<DateTime<Utc>>, GreedError> {
    let latest = now - Duration::minutes(RECENT_DATA_BUFFER_MINUTES);
    let end = range.end.min(latest);
    if range.start >= end {
        return Err(GreedError::new(&format!(
            "nothing to backfill: the range must start before {latest} \
             (data from the last {RECENT_DATA_BUFFER_MINUTES} minutes isn't backfilled)"
        )));
    }
    Ok(range.start..end)
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

    fn at(year: i32, month: u32, day: u32, hour: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(year, month, day, hour, 0, 0).unwrap()
    }

    fn bars_at(times: &[DateTime<Utc>]) -> Bars {
        Bars {
            symbol: AssetSymbol::new("VTI"),
            bars: times
                .iter()
                .map(|time| Bar {
                    timestamp: *time,
                    open: 1.0,
                    close: 2.0,
                    high: 3.0,
                    low: 0.5,
                    volume: 10,
                })
                .collect(),
        }
    }

    fn temp_db() -> PathBuf {
        let path = std::env::temp_dir().join(format!("greed_test_{}.duckdb", Uuid::new_v4()));
        bootstrap_at(&path).expect("bootstrap should succeed");
        path
    }

    fn count(path: &PathBuf, table: &str) -> i64 {
        Connection::open(path)
            .expect("db should open")
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                row.get(0)
            })
            .expect("table should be queryable")
    }

    fn vti() -> Vec<AssetSymbol> {
        vec![AssetSymbol::new("VTI")]
    }

    #[tokio::test]
    async fn requests_one_minute_bars_for_each_month_window() {
        let path = temp_db();
        let platform = Arc::new(MockPlatform::new());
        let backfiller = Backfiller::new(platform.clone(), path.clone());

        backfiller
            .backfill(&vti(), at(2026, 1, 15, 0)..at(2026, 3, 10, 0))
            .await
            .expect("backfill should succeed");

        let requests = platform.bar_requests();
        let windows: Vec<_> = requests.iter().map(|r| r.start..r.end).collect();
        assert_eq!(
            windows,
            vec![
                at(2026, 1, 15, 0)..at(2026, 2, 1, 0),
                at(2026, 2, 1, 0)..at(2026, 3, 1, 0),
                at(2026, 3, 1, 0)..at(2026, 3, 10, 0),
            ]
        );
        assert!(requests.iter().all(|r| r.timeframe == TimeFrame::OneMinute
            && r.limit.is_none()
            && r.symbol == AssetSymbol::new("VTI")));
        std::fs::remove_file(&path).ok();
    }

    #[tokio::test]
    async fn saves_bars_and_a_coverage_row_per_window() {
        let path = temp_db();
        let platform = Arc::new(MockPlatform::new().with_bars_results(vec![
            Ok(bars_at(&[at(2026, 1, 20, 14), at(2026, 1, 20, 15)])),
            Ok(bars_at(&[at(2026, 2, 3, 14)])),
        ]));
        let backfiller = Backfiller::new(platform, path.clone());

        let report = backfiller
            .backfill(&vti(), at(2026, 1, 15, 0)..at(2026, 2, 10, 0))
            .await
            .expect("backfill should succeed");

        assert_eq!(count(&path, "stock_bars"), 3);
        assert_eq!(count(&path, "backfill_coverage"), 2);
        assert_eq!(report.symbols.len(), 1);
        assert_eq!(report.symbols[0].windows, 2);
        assert_eq!(report.symbols[0].bars_fetched, 3);
        assert_eq!(report.symbols[0].bars_inserted, 3);
        std::fs::remove_file(&path).ok();
    }

    #[tokio::test]
    async fn an_empty_window_is_still_recorded_as_covered() {
        let path = temp_db();
        let backfiller = Backfiller::new(Arc::new(MockPlatform::new()), path.clone());

        backfiller
            .backfill(&vti(), at(2026, 1, 1, 0)..at(2026, 1, 5, 0))
            .await
            .expect("backfill should succeed");

        assert_eq!(count(&path, "stock_bars"), 0);
        assert_eq!(count(&path, "backfill_coverage"), 1);
        std::fs::remove_file(&path).ok();
    }

    #[tokio::test]
    async fn running_the_same_backfill_twice_does_not_duplicate_bars() {
        let path = temp_db();
        let bars = bars_at(&[at(2026, 1, 20, 14), at(2026, 1, 20, 15)]);
        let platform = Arc::new(MockPlatform::new().with_bars(bars));
        let backfiller = Backfiller::new(platform, path.clone());
        let range = at(2026, 1, 15, 0)..at(2026, 1, 25, 0);

        backfiller
            .backfill(&vti(), range.clone())
            .await
            .expect("first");
        let second = backfiller.backfill(&vti(), range).await.expect("second");

        assert_eq!(count(&path, "stock_bars"), 2);
        assert_eq!(second.total_inserted(), 0);
        std::fs::remove_file(&path).ok();
    }

    #[tokio::test]
    async fn a_second_identical_backfill_makes_no_platform_calls() {
        let path = temp_db();
        let platform = Arc::new(MockPlatform::new());
        let backfiller = Backfiller::new(platform.clone(), path.clone());
        let range = at(2026, 1, 15, 0)..at(2026, 3, 10, 0);

        backfiller
            .backfill(&vti(), range.clone())
            .await
            .expect("first");
        let calls_after_first = platform.bar_requests().len();
        let second = backfiller.backfill(&vti(), range).await.expect("second");

        assert_eq!(calls_after_first, 3);
        assert_eq!(platform.bar_requests().len(), calls_after_first);
        assert!(second.symbols[0].already_complete);
        std::fs::remove_file(&path).ok();
    }

    #[tokio::test]
    async fn extending_the_range_fetches_only_the_new_tail() {
        let path = temp_db();
        let platform = Arc::new(MockPlatform::new());
        let backfiller = Backfiller::new(platform.clone(), path.clone());
        backfiller
            .backfill(&vti(), at(2026, 1, 1, 0)..at(2026, 1, 10, 0))
            .await
            .expect("first");

        backfiller
            .backfill(&vti(), at(2026, 1, 1, 0)..at(2026, 1, 20, 0))
            .await
            .expect("second");

        let windows: Vec<_> = platform
            .bar_requests()
            .iter()
            .map(|r| r.start..r.end)
            .collect();
        assert_eq!(
            windows,
            vec![
                at(2026, 1, 1, 0)..at(2026, 1, 10, 0),
                at(2026, 1, 10, 0)..at(2026, 1, 20, 0),
            ]
        );
        std::fs::remove_file(&path).ok();
    }

    #[tokio::test]
    async fn a_hole_in_the_middle_fetches_only_the_hole() {
        let path = temp_db();
        let platform = Arc::new(MockPlatform::new());
        let backfiller = Backfiller::new(platform.clone(), path.clone());
        backfiller
            .backfill(&vti(), at(2026, 1, 1, 0)..at(2026, 1, 5, 0))
            .await
            .expect("head");
        backfiller
            .backfill(&vti(), at(2026, 1, 20, 0)..at(2026, 1, 25, 0))
            .await
            .expect("tail");
        let calls = platform.bar_requests().len();

        backfiller
            .backfill(&vti(), at(2026, 1, 1, 0)..at(2026, 1, 25, 0))
            .await
            .expect("whole range");

        let requests = platform.bar_requests();
        assert_eq!(requests.len(), calls + 1);
        let hole = requests.last().unwrap();
        assert_eq!(hole.start..hole.end, at(2026, 1, 5, 0)..at(2026, 1, 20, 0));
        std::fs::remove_file(&path).ok();
    }

    #[tokio::test]
    async fn overlapping_backfills_leave_one_row_per_bar() {
        let path = temp_db();
        let bars = bars_at(&[at(2026, 1, 8, 14), at(2026, 1, 12, 14)]);
        let platform = Arc::new(MockPlatform::new().with_bars(bars));
        let backfiller = Backfiller::new(platform, path.clone());

        backfiller
            .backfill(&vti(), at(2026, 1, 1, 0)..at(2026, 1, 10, 0))
            .await
            .expect("first");
        backfiller
            .backfill(&vti(), at(2026, 1, 5, 0)..at(2026, 1, 15, 0))
            .await
            .expect("second");

        assert_eq!(count(&path, "stock_bars"), 2);
        std::fs::remove_file(&path).ok();
    }

    #[tokio::test]
    async fn a_failed_window_stops_the_run_but_keeps_earlier_windows() {
        let path = temp_db();
        let platform = Arc::new(MockPlatform::new().with_bars_results(vec![
            Ok(bars_at(&[at(2026, 1, 20, 14)])),
            Err("api exploded".to_string()),
        ]));
        let backfiller = Backfiller::new(platform.clone(), path.clone());

        let result = backfiller
            .backfill(&vti(), at(2026, 1, 15, 0)..at(2026, 3, 10, 0))
            .await;

        let error = result.expect_err("second window should fail").to_string();
        assert!(error.contains("VTI"), "{error}");
        assert!(error.contains("api exploded"), "{error}");
        assert_eq!(platform.bar_requests().len(), 2);
        assert_eq!(count(&path, "stock_bars"), 1);
        assert_eq!(count(&path, "backfill_coverage"), 1);
        std::fs::remove_file(&path).ok();
    }

    #[tokio::test]
    async fn backfills_each_symbol_once_even_when_listed_twice() {
        let path = temp_db();
        let platform = Arc::new(MockPlatform::new());
        let backfiller = Backfiller::new(platform.clone(), path.clone());
        let symbols = vec![
            AssetSymbol::new("VTI"),
            AssetSymbol::new("SPY"),
            AssetSymbol::new("VTI"),
        ];

        let report = backfiller
            .backfill(&symbols, at(2026, 1, 1, 0)..at(2026, 1, 5, 0))
            .await
            .expect("backfill should succeed");

        let requested: Vec<_> = platform
            .bar_requests()
            .into_iter()
            .map(|r| r.symbol.symbol)
            .collect();
        assert_eq!(requested, vec!["VTI", "SPY"]);
        assert_eq!(report.symbols.len(), 2);
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn available_range_leaves_a_range_in_the_past_alone() {
        let range = at(2026, 1, 1, 0)..at(2026, 1, 2, 0);
        let result = available_range(range.clone(), at(2026, 6, 1, 0)).expect("in the past");
        assert_eq!(result, range);
    }

    #[test]
    fn available_range_trims_the_recent_tail() {
        let now = at(2026, 6, 1, 12);
        let result = available_range(at(2026, 5, 1, 0)..now, now).expect("trimmed");
        assert_eq!(result.end, now - Duration::minutes(15));
    }

    #[test]
    fn available_range_errors_when_nothing_is_left() {
        let now = at(2026, 6, 1, 12);
        let start = now - Duration::minutes(5);
        assert!(available_range(start..now, now).is_err());
    }
}
