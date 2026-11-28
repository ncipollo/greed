mod html;
mod writer;

pub use writer::{write, TraceFormat};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// Everything a backtest did, for inspection after the fact. Laid out as tables so it serializes
/// to both JSON and TOML (TOML has no top-level arrays and no `null`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BacktestTrace {
    pub summary: TraceSummary,
    pub events: Vec<TraceEvent>,
    /// Portfolio value over time, one point per simulated day plus the end.
    pub equity: Vec<EquityPoint>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TraceSummary {
    pub config: String,
    pub range_start: DateTime<Utc>,
    pub range_end: DateTime<Utc>,
    pub starting_cash: f64,
    pub final_value: f64,
    pub return_percent: f64,
    pub trade_count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    Trade,
    RejectedOrder,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraceSide {
    Buy,
    Sell,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TraceEvent {
    pub time: DateTime<Utc>,
    pub kind: EventKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strategy: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tactic: Option<String>,
    pub symbol: String,
    pub side: TraceSide,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quantity: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub price: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notional: Option<f64>,
    /// The fee charged on a fill, when there was one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fee: Option<f64>,
    /// What slippage cost on a fill (price moved against the trader times quantity), when any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slippage: Option<f64>,
    /// Cash once the event is done.
    pub cash_after: f64,
    /// Cash plus holdings once the event is done.
    pub equity_after: f64,
    /// Why an order was rejected.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EquityPoint {
    pub time: DateTime<Utc>,
    pub cash: f64,
    pub equity: f64,
}

/// Collects events while a simulation runs. Cheap to clone; clones share the same log.
#[derive(Clone, Debug, Default)]
pub struct TraceRecorder {
    log: Arc<Mutex<Log>>,
}

#[derive(Debug, Default)]
struct Log {
    events: Vec<TraceEvent>,
    equity: Vec<EquityPoint>,
}

impl TraceRecorder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_event(&self, event: TraceEvent) {
        self.log.lock().unwrap().events.push(event);
    }

    pub fn record_equity(&self, point: EquityPoint) {
        self.log.lock().unwrap().equity.push(point);
    }

    pub fn build_trace(&self, summary: TraceSummary) -> BacktestTrace {
        let log = self.log.lock().unwrap();
        BacktestTrace {
            summary,
            events: log.events.clone(),
            equity: log.equity.clone(),
        }
    }
}

#[cfg(test)]
pub(crate) mod fixture {
    use super::*;
    use chrono::TimeZone;

    pub fn trace() -> BacktestTrace {
        let at = |hour: u32| Utc.with_ymd_and_hms(2026, 1, 5, hour, 30, 0).unwrap();
        BacktestTrace {
            summary: TraceSummary {
                config: "greed.toml".to_string(),
                range_start: at(14),
                range_end: at(15),
                starting_cash: 100_000.0,
                final_value: 101_000.0,
                return_percent: 1.0,
                trade_count: 2,
            },
            events: vec![
                TraceEvent {
                    time: at(14),
                    kind: EventKind::Trade,
                    strategy: Some("core".to_string()),
                    tactic: Some("ETF".to_string()),
                    symbol: "VTI".to_string(),
                    side: TraceSide::Buy,
                    quantity: Some(100.0),
                    price: Some(100.0),
                    notional: Some(10_000.0),
                    fee: None,
                    slippage: None,
                    cash_after: 90_000.0,
                    equity_after: 100_000.0,
                    reason: None,
                },
                TraceEvent {
                    time: at(15),
                    kind: EventKind::RejectedOrder,
                    strategy: None,
                    tactic: None,
                    symbol: "SPY".to_string(),
                    side: TraceSide::Sell,
                    quantity: Some(1.0),
                    price: None,
                    notional: None,
                    fee: None,
                    slippage: None,
                    cash_after: 90_000.0,
                    equity_after: 101_000.0,
                    reason: Some("can't sell 1 of SPY: only 0 held".to_string()),
                },
            ],
            equity: vec![
                EquityPoint {
                    time: at(14),
                    cash: 90_000.0,
                    equity: 100_000.0,
                },
                EquityPoint {
                    time: at(15),
                    cash: 90_000.0,
                    equity: 101_000.0,
                },
            ],
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn the_recorder_shares_its_log_between_clones() {
        let recorder = TraceRecorder::new();
        let clone = recorder.clone();
        let trace = fixture::trace();

        clone.record_event(trace.events[0].clone());
        recorder.record_equity(trace.equity[0].clone());

        let built = recorder.build_trace(trace.summary.clone());
        assert_eq!(built.events, vec![trace.events[0].clone()]);
        assert_eq!(built.equity, vec![trace.equity[0].clone()]);
    }
}
