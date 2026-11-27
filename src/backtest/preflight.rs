use crate::analysis::time_range::MAX_LOOKBACK_DAYS;
use crate::asset::AssetSymbol;
use crate::backfill::{available_range, Backfiller, CoverageChecker};
use crate::db;
use crate::error::GreedError;
use crate::platform::bar::time_frame::TimeFrame;
use crate::platform::FinancialPlatform;
use chrono::{DateTime, Duration, Utc};
use log::info;
use std::ops::Range;
use std::path::Path;
use std::sync::Arc;

/// Makes sure the database holds minute bars for everything the simulation will read: the
/// simulated range plus the look-back the tactics' bar requests reach into before it.
///
/// This is the only phase of a backtest that may use the network or write to the database. The
/// live platform is built by `make_platform` only if something is actually missing, and it is
/// dropped before this returns. Returns the range that is now covered.
pub(crate) async fn ensure_data<F>(
    db_path: &Path,
    symbols: &[AssetSymbol],
    simulated: &Range<DateTime<Utc>>,
    now: DateTime<Utc>,
    make_platform: F,
) -> Result<Range<DateTime<Utc>>, GreedError>
where
    F: FnOnce() -> Result<Arc<dyn FinancialPlatform>, GreedError>,
{
    let lookback = Duration::days(MAX_LOOKBACK_DAYS);
    let needed = available_range((simulated.start - lookback)..simulated.end, now)?;
    let plan =
        CoverageChecker::plan(db_path, symbols, &TimeFrame::OneMinute, needed.clone()).await?;
    if plan.is_complete() {
        return Ok(needed);
    }
    info!("📥 stored data is incomplete for this backtest, backfilling before simulating");
    // Create the file and schema up front; each chunk then holds the write lock only briefly.
    db::bootstrap_at_with_retry(db_path).await?;
    let platform = make_platform()?;
    Backfiller::new(platform, db_path.to_path_buf())
        .backfill(symbols, needed.clone())
        .await?;
    Ok(needed)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::db::bootstrap_at;
    use crate::db::stock_bars;
    use crate::platform::MockPlatform;
    use chrono::TimeZone;
    use uuid::Uuid;

    fn at(day: u32, hour: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 1, day, hour, 0, 0).unwrap()
    }

    fn vti() -> Vec<AssetSymbol> {
        vec![AssetSymbol::new("VTI")]
    }

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 6, 1, 0, 0, 0).unwrap()
    }

    fn temp_db() -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!("greed_preflight_{}.duckdb", Uuid::new_v4()));
        bootstrap_at(&path).unwrap();
        path
    }

    fn mark_covered(path: &Path, window: &Range<DateTime<Utc>>) {
        let mut connection = db::open_for_write(path).unwrap();
        stock_bars::insert_chunk(
            &mut connection,
            &AssetSymbol::new("VTI"),
            &TimeFrame::OneMinute,
            window,
            &[],
        )
        .unwrap();
    }

    #[tokio::test]
    async fn complete_coverage_never_builds_a_platform() {
        let path = temp_db();
        let simulated = at(10, 0)..at(12, 0);
        mark_covered(
            &path,
            &(at(10, 0) - Duration::days(MAX_LOOKBACK_DAYS)..at(12, 0)),
        );

        let needed = ensure_data(&path, &vti(), &simulated, now(), || {
            panic!("a live platform must not be built when the data is already stored")
        })
        .await
        .unwrap();

        assert_eq!(needed.end, at(12, 0));
        std::fs::remove_file(&path).ok();
    }

    #[tokio::test]
    async fn missing_data_is_backfilled_through_the_platform() {
        let path = temp_db();
        let platform = Arc::new(MockPlatform::new());
        let simulated = at(10, 0)..at(12, 0);

        let for_factory = platform.clone();
        ensure_data(&path, &vti(), &simulated, now(), move || {
            Ok(for_factory as Arc<dyn FinancialPlatform>)
        })
        .await
        .unwrap();

        assert!(!platform.bar_requests().is_empty());
        let again = ensure_data(&path, &vti(), &simulated, now(), || {
            panic!("second run should find the data already stored")
        })
        .await;
        assert!(again.is_ok());
        std::fs::remove_file(&path).ok();
    }

    #[tokio::test]
    async fn a_missing_database_file_is_created_by_the_backfill() {
        let path = std::env::temp_dir().join(format!("greed_fresh_{}.duckdb", Uuid::new_v4()));
        let simulated = at(10, 0)..at(12, 0);

        ensure_data(&path, &vti(), &simulated, now(), || {
            Ok(Arc::new(MockPlatform::new()) as Arc<dyn FinancialPlatform>)
        })
        .await
        .unwrap();

        assert!(path.exists());
        std::fs::remove_file(&path).ok();
    }
}
