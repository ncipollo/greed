use crate::asset::AssetSymbol;
use chrono::{DateTime, Utc};
use std::fmt::{Display, Formatter};
use std::ops::Range;

/// What a backfill fetched and saved for one symbol.
#[derive(Clone, Debug, PartialEq)]
pub struct SymbolReport {
    pub symbol: AssetSymbol,
    pub windows: usize,
    pub bars_fetched: usize,
    pub bars_inserted: usize,
    /// True when the database already covered the whole range.
    pub already_complete: bool,
}

impl SymbolReport {
    pub fn new(symbol: AssetSymbol) -> Self {
        Self {
            symbol,
            already_complete: false,
            windows: 0,
            bars_fetched: 0,
            bars_inserted: 0,
        }
    }

    pub fn record_window(&mut self, fetched: usize, inserted: usize) {
        self.windows += 1;
        self.bars_fetched += fetched;
        self.bars_inserted += inserted;
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BackfillReport {
    pub range: Range<DateTime<Utc>>,
    pub symbols: Vec<SymbolReport>,
}

impl BackfillReport {
    pub fn total_inserted(&self) -> usize {
        self.symbols.iter().map(|s| s.bars_inserted).sum()
    }
}

impl Display for BackfillReport {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let format = "%Y-%m-%d %H:%M UTC";
        writeln!(
            f,
            "Backfilled {} to {}",
            self.range.start.format(format),
            self.range.end.format(format)
        )?;
        for symbol in &self.symbols {
            if symbol.already_complete {
                writeln!(f, "  {}: already complete, skipped", symbol.symbol.symbol)?;
                continue;
            }
            writeln!(
                f,
                "  {}: {} bars fetched, {} new, {} windows",
                symbol.symbol.symbol, symbol.bars_fetched, symbol.bars_inserted, symbol.windows
            )?;
        }
        write!(f, "Saved {} new bars", self.total_inserted())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn record_window_accumulates() {
        let mut report = SymbolReport::new(AssetSymbol::new("VTI"));

        report.record_window(10, 4);
        report.record_window(5, 5);

        assert_eq!(report.windows, 2);
        assert_eq!(report.bars_fetched, 15);
        assert_eq!(report.bars_inserted, 9);
    }

    #[test]
    fn display_marks_complete_symbols_as_skipped() {
        let mut vti = SymbolReport::new(AssetSymbol::new("VTI"));
        vti.already_complete = true;
        let report = BackfillReport {
            range: Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap()
                ..Utc.with_ymd_and_hms(2026, 2, 1, 0, 0, 0).unwrap(),
            symbols: vec![vti],
        };

        assert!(report
            .to_string()
            .contains("VTI: already complete, skipped"));
    }

    #[test]
    fn display_lists_each_symbol_and_the_total() {
        let mut vti = SymbolReport::new(AssetSymbol::new("VTI"));
        vti.record_window(10, 4);
        let mut spy = SymbolReport::new(AssetSymbol::new("SPY"));
        spy.record_window(3, 3);
        let report = BackfillReport {
            range: Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap()
                ..Utc.with_ymd_and_hms(2026, 2, 1, 0, 0, 0).unwrap(),
            symbols: vec![vti, spy],
        };

        let text = report.to_string();

        assert_eq!(
            text,
            "Backfilled 2026-01-01 00:00 UTC to 2026-02-01 00:00 UTC\n\
             \x20 VTI: 10 bars fetched, 4 new, 1 windows\n\
             \x20 SPY: 3 bars fetched, 3 new, 1 windows\n\
             Saved 7 new bars"
        );
    }
}
