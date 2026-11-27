use crate::asset::AssetSymbol;
use chrono::{DateTime, NaiveDate, Utc};
use std::fmt::{Display, Formatter};
use std::ops::Range;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// A stage of `backfill` or `backtest`, announced as it begins.
#[derive(Clone, Debug, PartialEq)]
pub enum Phase {
    CheckingCoverage,
    /// Fetching missing data for this many symbols.
    Backfilling {
        symbols: usize,
    },
    LoadingData,
    Simulating,
    WritingTrace(PathBuf),
}

impl Display for Phase {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CheckingCoverage => write!(f, "Checking data coverage"),
            Self::Backfilling { symbols } => write!(f, "Backfilling {symbols} symbol(s)"),
            Self::LoadingData => write!(f, "Loading data"),
            Self::Simulating => write!(f, "Simulating"),
            Self::WritingTrace(path) => write!(f, "Writing trace to {}", path.display()),
        }
    }
}

/// What a backfill is about to do for one symbol. No windows means the database
/// already covers the range.
#[derive(Clone, Debug, PartialEq)]
pub struct SymbolPlan {
    pub symbol: AssetSymbol,
    pub first_month: NaiveDate,
    pub last_month: NaiveDate,
    pub windows: usize,
}

/// What one finished month holds.
#[derive(Clone, Debug, PartialEq)]
pub enum MonthDetail {
    /// A backfill saved this many bars for the month.
    Bars(usize),
    /// A simulated month ended with this portfolio value after this many trades.
    Simulated { equity: f64, trades: usize },
}

#[derive(Clone, Debug, PartialEq)]
pub struct MonthProgress {
    /// The symbol being backfilled; none for a whole-portfolio simulation.
    pub symbol: Option<AssetSymbol>,
    /// First day of the month.
    pub month: NaiveDate,
    pub detail: MonthDetail,
}

/// One fetched window of a backfill. Verbose only.
#[derive(Clone, Debug, PartialEq)]
pub struct ChunkProgress {
    pub symbol: AssetSymbol,
    pub window: Range<DateTime<Utc>>,
    pub fetched: usize,
    pub inserted: usize,
}

/// One simulated interval. Verbose only.
#[derive(Clone, Debug, PartialEq)]
pub struct TickProgress {
    pub time: DateTime<Utc>,
    pub equity: f64,
    pub trades: usize,
}

/// Receives progress as long-running commands work. Core logic only emits events; the CLI
/// decides how to show them. Every method defaults to doing nothing.
pub trait ProgressReporter: Send + Sync {
    fn phase(&self, _phase: Phase) {}
    fn symbol_plan(&self, _plan: SymbolPlan) {}
    fn month_done(&self, _month: MonthProgress) {}
    fn chunk(&self, _chunk: ChunkProgress) {}
    fn tick(&self, _tick: TickProgress) {}
}

/// Ignores everything.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopReporter;

impl ProgressReporter for NoopReporter {}

/// Everything a reporter was told, in order.
#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    Phase(Phase),
    SymbolPlan(SymbolPlan),
    Month(MonthProgress),
    Chunk(ChunkProgress),
    Tick(TickProgress),
}

/// Remembers events so tests can assert on the sequence.
#[derive(Debug, Default)]
pub struct RecordingReporter {
    events: Mutex<Vec<Event>>,
}

impl RecordingReporter {
    pub fn events(&self) -> Vec<Event> {
        self.events.lock().unwrap().clone()
    }

    fn push(&self, event: Event) {
        self.events.lock().unwrap().push(event);
    }
}

impl ProgressReporter for RecordingReporter {
    fn phase(&self, phase: Phase) {
        self.push(Event::Phase(phase));
    }
    fn symbol_plan(&self, plan: SymbolPlan) {
        self.push(Event::SymbolPlan(plan));
    }
    fn month_done(&self, month: MonthProgress) {
        self.push(Event::Month(month));
    }
    fn chunk(&self, chunk: ChunkProgress) {
        self.push(Event::Chunk(chunk));
    }
    fn tick(&self, tick: TickProgress) {
        self.push(Event::Tick(tick));
    }
}

/// A reporter plus whether to emit the detailed (per chunk, per tick) events. Passed down
/// through backfill and backtest so verbosity is decided once, at the edge.
#[derive(Clone)]
pub struct Progress {
    reporter: Arc<dyn ProgressReporter>,
    verbose: bool,
}

impl Progress {
    pub fn new(reporter: Arc<dyn ProgressReporter>, verbose: bool) -> Self {
        Self { reporter, verbose }
    }

    /// Reports nothing.
    pub fn quiet() -> Self {
        Self::new(Arc::new(NoopReporter), false)
    }

    /// A recording reporter for tests, plus a handle to read its events.
    pub fn recording(verbose: bool) -> (Self, Arc<RecordingReporter>) {
        let recorder = Arc::new(RecordingReporter::default());
        (Self::new(recorder.clone(), verbose), recorder)
    }

    pub fn is_verbose(&self) -> bool {
        self.verbose
    }

    pub fn phase(&self, phase: Phase) {
        self.reporter.phase(phase);
    }

    pub fn symbol_plan(&self, plan: SymbolPlan) {
        self.reporter.symbol_plan(plan);
    }

    pub fn month_done(&self, month: MonthProgress) {
        self.reporter.month_done(month);
    }

    /// Only sent when verbose.
    pub fn chunk(&self, chunk: ChunkProgress) {
        if self.verbose {
            self.reporter.chunk(chunk);
        }
    }

    /// Only sent when verbose.
    pub fn tick(&self, tick: TickProgress) {
        if self.verbose {
            self.reporter.tick(tick);
        }
    }
}

impl Default for Progress {
    fn default() -> Self {
        Self::quiet()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::TimeZone;

    fn tick() -> TickProgress {
        TickProgress {
            time: Utc.with_ymd_and_hms(2026, 1, 5, 14, 30, 0).unwrap(),
            equity: 1.0,
            trades: 0,
        }
    }

    #[test]
    fn detailed_events_are_dropped_unless_verbose() {
        let (quiet, quiet_events) = Progress::recording(false);
        let (loud, loud_events) = Progress::recording(true);

        quiet.tick(tick());
        loud.tick(tick());
        quiet.phase(Phase::Simulating);

        assert_eq!(quiet_events.events(), vec![Event::Phase(Phase::Simulating)]);
        assert_eq!(loud_events.events(), vec![Event::Tick(tick())]);
    }

    #[test]
    fn phases_have_readable_names() {
        assert_eq!(
            Phase::CheckingCoverage.to_string(),
            "Checking data coverage"
        );
        assert_eq!(
            Phase::Backfilling { symbols: 2 }.to_string(),
            "Backfilling 2 symbol(s)"
        );
        assert_eq!(
            Phase::WritingTrace(PathBuf::from("out.json")).to_string(),
            "Writing trace to out.json"
        );
    }
}
