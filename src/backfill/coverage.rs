use crate::asset::AssetSymbol;
use crate::db;
use crate::db::backfill_coverage;
use crate::error::GreedError;
use crate::interval_set;
use crate::platform::bar::time_frame::TimeFrame;
use chrono::{DateTime, Utc};
use itertools::Itertools;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::ops::Range;
use std::path::Path;

type Gaps = HashMap<AssetSymbol, Vec<Range<DateTime<Utc>>>>;

/// What a backfill would still need to fetch, per symbol. Symbols with full
/// coverage have an empty gap list.
#[derive(Clone, Debug, PartialEq)]
pub struct BackfillPlan {
    pub range: Range<DateTime<Utc>>,
    pub per_symbol_gaps: Gaps,
}

impl BackfillPlan {
    /// True only when every symbol has coverage for the whole range.
    pub fn is_complete(&self) -> bool {
        self.per_symbol_gaps.values().all(|gaps| gaps.is_empty())
    }

    pub fn gaps_for(&self, symbol: &AssetSymbol) -> &[Range<DateTime<Utc>>] {
        self.per_symbol_gaps
            .get(symbol)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }
}

impl Display for BackfillPlan {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let format = "%Y-%m-%d %H:%M UTC";
        writeln!(
            f,
            "Backfill plan for {} to {}",
            self.range.start.format(format),
            self.range.end.format(format)
        )?;
        for symbol in self
            .per_symbol_gaps
            .keys()
            .sorted_by_key(|s| s.symbol.clone())
        {
            let gaps = self.gaps_for(symbol);
            if gaps.is_empty() {
                writeln!(f, "  {}: complete", symbol.symbol)?;
                continue;
            }
            let missing: chrono::Duration = gaps.iter().map(|g| g.end - g.start).sum();
            writeln!(
                f,
                "  {}: missing {} minutes in {} windows",
                symbol.symbol,
                missing.num_minutes(),
                gaps.len()
            )?;
            for gap in gaps {
                writeln!(
                    f,
                    "    {} to {}",
                    gap.start.format(format),
                    gap.end.format(format)
                )?;
            }
        }
        if self.is_complete() {
            write!(f, "Nothing to fetch")
        } else {
            write!(f, "Dry run: nothing was fetched")
        }
    }
}

/// Works out which parts of a range the local database hasn't covered yet,
/// using the backfill ledger (not bar counts, so market closures never look
/// like missing data). Read-only.
pub struct CoverageChecker;

impl CoverageChecker {
    /// Gaps per symbol for `range`. Every requested symbol gets an entry.
    pub fn missing_ranges(
        connection: &duckdb::Connection,
        symbols: &[AssetSymbol],
        timeframe: &TimeFrame,
        range: &Range<DateTime<Utc>>,
    ) -> Result<Gaps, GreedError> {
        let mut gaps = HashMap::new();
        for symbol in symbols.iter().unique() {
            let covered = backfill_coverage::overlapping(connection, symbol, timeframe, range)?;
            gaps.insert(symbol.clone(), interval_set::subtract(range, covered));
        }
        Ok(gaps)
    }

    /// Builds a plan from the database at `db_path`. A missing file means
    /// nothing is covered.
    pub async fn plan(
        db_path: &Path,
        symbols: &[AssetSymbol],
        timeframe: &TimeFrame,
        range: Range<DateTime<Utc>>,
    ) -> Result<BackfillPlan, GreedError> {
        let per_symbol_gaps = match db::open_read_only_with_retry(db_path).await? {
            Some(connection) => Self::missing_ranges(&connection, symbols, timeframe, &range)?,
            None => symbols
                .iter()
                .unique()
                .map(|s| (s.clone(), interval_set::subtract(&range, vec![])))
                .collect(),
        };
        Ok(BackfillPlan {
            range,
            per_symbol_gaps,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::db::migrations;
    use chrono::TimeZone;
    use duckdb::Connection;

    fn at(day: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 1, day, 0, 0, 0).unwrap()
    }

    fn connection_with(rows: &[(&str, Range<DateTime<Utc>>)]) -> Connection {
        let connection = Connection::open_in_memory().unwrap();
        migrations::apply(&connection).unwrap();
        for (symbol, window) in rows {
            backfill_coverage::insert(
                &connection,
                &AssetSymbol::new(symbol),
                &TimeFrame::OneMinute,
                window,
            )
            .unwrap();
        }
        connection
    }

    fn missing(connection: &Connection, symbols: &[&str], range: Range<DateTime<Utc>>) -> Gaps {
        let symbols: Vec<_> = symbols.iter().map(|s| AssetSymbol::new(s)).collect();
        CoverageChecker::missing_ranges(connection, &symbols, &TimeFrame::OneMinute, &range)
            .unwrap()
    }

    #[test]
    fn fully_covered_has_no_gaps() {
        let connection = connection_with(&[("VTI", at(1)..at(10))]);

        let gaps = missing(&connection, &["VTI"], at(2)..at(9));

        assert!(gaps[&AssetSymbol::new("VTI")].is_empty());
    }

    #[test]
    fn fully_missing_returns_the_whole_range() {
        let connection = connection_with(&[]);

        let gaps = missing(&connection, &["VTI"], at(2)..at(9));

        assert_eq!(gaps[&AssetSymbol::new("VTI")], vec![at(2)..at(9)]);
    }

    #[test]
    fn hole_in_the_middle_is_reported() {
        let connection = connection_with(&[("VTI", at(1)..at(4)), ("VTI", at(6)..at(10))]);

        let gaps = missing(&connection, &["VTI"], at(1)..at(10));

        assert_eq!(gaps[&AssetSymbol::new("VTI")], vec![at(4)..at(6)]);
    }

    #[test]
    fn coverage_overlapping_the_edges_is_trimmed() {
        let connection = connection_with(&[("VTI", at(1)..at(5)), ("VTI", at(8)..at(20))]);

        let gaps = missing(&connection, &["VTI"], at(3)..at(10));

        assert_eq!(gaps[&AssetSymbol::new("VTI")], vec![at(5)..at(8)]);
    }

    #[test]
    fn adjacent_ledger_rows_merge() {
        let connection = connection_with(&[("VTI", at(1)..at(5)), ("VTI", at(5)..at(10))]);

        let gaps = missing(&connection, &["VTI"], at(1)..at(10));

        assert!(gaps[&AssetSymbol::new("VTI")].is_empty());
    }

    #[test]
    fn symbols_have_independent_gaps() {
        let connection = connection_with(&[("VTI", at(1)..at(10)), ("SPY", at(1)..at(5))]);

        let gaps = missing(&connection, &["VTI", "SPY"], at(1)..at(10));

        assert!(gaps[&AssetSymbol::new("VTI")].is_empty());
        assert_eq!(gaps[&AssetSymbol::new("SPY")], vec![at(5)..at(10)]);
    }

    #[test]
    fn plan_is_complete_only_when_every_symbol_is_covered() {
        let connection = connection_with(&[("VTI", at(1)..at(10)), ("SPY", at(1)..at(5))]);
        let range = at(1)..at(10);

        let partial = BackfillPlan {
            range: range.clone(),
            per_symbol_gaps: missing(&connection, &["VTI", "SPY"], range.clone()),
        };
        let complete = BackfillPlan {
            range: range.clone(),
            per_symbol_gaps: missing(&connection, &["VTI"], range),
        };

        assert!(!partial.is_complete());
        assert!(complete.is_complete());
    }

    #[tokio::test]
    async fn missing_database_file_means_everything_is_missing() {
        let path =
            std::env::temp_dir().join(format!("greed_missing_{}.duckdb", uuid::Uuid::new_v4()));

        let plan = CoverageChecker::plan(
            &path,
            &[AssetSymbol::new("VTI")],
            &TimeFrame::OneMinute,
            at(1)..at(3),
        )
        .await
        .unwrap();

        assert_eq!(plan.gaps_for(&AssetSymbol::new("VTI")), &[at(1)..at(3)]);
        assert!(!path.exists(), "read-only check must not create the file");
    }
}
