use crate::backtest::trace::{self, TraceAssumptions, TraceSummary};
use crate::backtest::{BacktestResult, PreparedBacktest};
use crate::clock::{Clock, Timing};
use crate::error::GreedError;
use crate::platform::FinancialPlatform;
use crate::progress::{MonthDetail, MonthProgress, Phase, Progress, TickProgress};
use crate::run::GreedRunner;
use chrono::{DateTime, Datelike, Duration as ChronoDuration, NaiveDate, Utc};
use log::warn;
use std::time::Duration;

/// A simulation tick finding no trades for this long means the market is closed (overnight,
/// weekend, holiday). Illiquid symbols can miss many minutes within a session, so this is
/// generous: only a real closure is skipped.
const MARKET_CLOSED_AFTER: ChronoDuration = ChronoDuration::minutes(15);

/// Replays the prepared range: steps the simulated clock from start to end at the config's
/// interval, running every strategy once per step against the prices of that moment.
///
/// Closed-market stretches are skipped by jumping the clock straight to the first tick at or
/// after the next bar. Live greed would keep ticking through them, but with no new prices every
/// rule would see the same data and trade nothing more, so the result is the same.
///
/// Simulation takes the concrete `BacktestPlatform` and no database path: it can neither reach a
/// live platform nor write anything down.
pub async fn simulate(
    prepared: PreparedBacktest,
    progress: &Progress,
) -> Result<BacktestResult, GreedError> {
    let PreparedBacktest {
        config,
        config_path,
        range,
        platform,
        clock,
        starting_cash,
        trace,
        ..
    } = prepared;
    if config.interval == 0 {
        return Err(GreedError::new(
            "the config's interval must be at least 1 second",
        ));
    }
    if config.interval < 60 {
        warn!(
            "interval is {}s but prices are minute bars; ticks inside a minute see the same price",
            config.interval
        );
    }
    let interval_seconds = config.interval;
    let step = ChronoDuration::seconds(config.interval as i64);
    let runner = GreedRunner::for_simulation(
        config,
        config_path.clone(),
        platform.clone(),
        Timing::simulated(clock.clone()),
    )
    .await?;

    progress.phase(Phase::Simulating);
    clock.set(range.start);
    let mut ticks = 0u64;
    let mut last_snapshot_day = None;
    let mut month = MonthTracker::default();
    while clock.now() < range.end {
        if let Some(done) = month.enter(clock.now(), platform.order_count()) {
            let equity = platform.account().await?.equity;
            progress.month_done(done.into_progress(equity));
        }
        if !platform.market_active_within(MARKET_CLOSED_AFTER) {
            let Some(next) = platform.next_bar_time() else {
                break;
            };
            clock.set(first_tick_at_or_after(range.start, step, next));
            continue;
        }
        platform.fill_due_orders();
        runner.evaluate_strategies(Duration::ZERO).await;
        ticks += 1;
        if progress.is_verbose() {
            progress.tick(TickProgress {
                time: clock.now(),
                equity: platform.account().await?.equity,
                trades: platform.order_count(),
            });
        }
        if trace.is_some() && last_snapshot_day != Some(clock.now().date_naive()) {
            platform.snapshot_equity();
            last_snapshot_day = Some(clock.now().date_naive());
        }
        clock.advance(step);
    }

    clock.set(range.end);
    let final_value = platform.account().await?.equity;
    let trade_count = platform.order_count();
    if let Some(done) = month.finish(trade_count) {
        progress.month_done(done.into_progress(final_value));
    }
    log::debug!("simulated {ticks} ticks");
    let result = BacktestResult::new(
        range,
        starting_cash,
        final_value,
        trade_count,
        platform.unfilled_orders(),
        platform.costs(),
    );
    if let Some((path, recorder)) = trace {
        progress.phase(Phase::WritingTrace(path.clone()));
        platform.snapshot_equity();
        let summary = TraceSummary {
            config: config_path.display().to_string(),
            range_start: result.range.start,
            range_end: result.range.end,
            starting_cash,
            final_value,
            return_percent: result.return_percent,
            trade_count,
        };
        let costs = platform.costs();
        let assumptions = TraceAssumptions {
            fill_rule: "next bar open".to_string(),
            order_types: "market only".to_string(),
            fee: costs.fee,
            slippage_bps: costs.slippage_bps,
            starting_cash,
            interval_seconds,
            market_closed_skip_minutes: MARKET_CLOSED_AFTER.num_minutes(),
            unfilled_orders: platform.unfilled_trace(),
        };
        trace::write(&path, &recorder.build_trace(summary, assumptions))?;
    }
    Ok(result)
}

/// A simulated month that just ended, waiting for its closing equity.
struct FinishedMonth {
    month: NaiveDate,
    trades: usize,
}

impl FinishedMonth {
    fn into_progress(self, equity: f64) -> MonthProgress {
        MonthProgress {
            symbol: None,
            month: self.month,
            detail: MonthDetail::Simulated {
                equity,
                trades: self.trades,
            },
        }
    }
}

/// Notices when the simulated clock moves into a new month so each month is reported once.
#[derive(Default)]
struct MonthTracker {
    current: Option<NaiveDate>,
    trades_at_start: usize,
}

impl MonthTracker {
    /// Call before each tick; returns the month that just ended when `now` is in a later one.
    fn enter(&mut self, now: DateTime<Utc>, trades_so_far: usize) -> Option<FinishedMonth> {
        let month = NaiveDate::from_ymd_opt(now.year(), now.month(), 1)?;
        match self.current {
            Some(current) if current == month => None,
            Some(current) => {
                let finished = FinishedMonth {
                    month: current,
                    trades: trades_so_far - self.trades_at_start,
                };
                self.current = Some(month);
                self.trades_at_start = trades_so_far;
                Some(finished)
            }
            None => {
                self.current = Some(month);
                self.trades_at_start = trades_so_far;
                None
            }
        }
    }

    /// The month in progress when the simulation ends.
    fn finish(&mut self, trades_so_far: usize) -> Option<FinishedMonth> {
        let month = self.current.take()?;
        Some(FinishedMonth {
            month,
            trades: trades_so_far - self.trades_at_start,
        })
    }
}

/// Ticks happen at `start + k * step`; this is the first one at or after `time`.
fn first_tick_at_or_after(
    start: chrono::DateTime<chrono::Utc>,
    step: ChronoDuration,
    time: chrono::DateTime<chrono::Utc>,
) -> chrono::DateTime<chrono::Utc> {
    let step_ms = step.num_milliseconds().max(1);
    let behind_ms = (time - start).num_milliseconds().max(0);
    let ticks = (behind_ms + step_ms - 1) / step_ms;
    start + ChronoDuration::milliseconds(ticks * step_ms)
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::{TimeZone, Utc};

    #[test]
    fn first_tick_rounds_up_to_the_step_grid() {
        let start = Utc.with_ymd_and_hms(2026, 1, 5, 14, 30, 0).unwrap();
        let step = ChronoDuration::minutes(5);

        let tick = first_tick_at_or_after(
            start,
            step,
            Utc.with_ymd_and_hms(2026, 1, 5, 14, 41, 0).unwrap(),
        );

        assert_eq!(tick, Utc.with_ymd_and_hms(2026, 1, 5, 14, 45, 0).unwrap());
    }

    #[test]
    fn first_tick_keeps_an_exact_hit() {
        let start = Utc.with_ymd_and_hms(2026, 1, 5, 14, 30, 0).unwrap();

        let tick = first_tick_at_or_after(
            start,
            ChronoDuration::minutes(1),
            Utc.with_ymd_and_hms(2026, 1, 5, 14, 35, 0).unwrap(),
        );

        assert_eq!(tick, Utc.with_ymd_and_hms(2026, 1, 5, 14, 35, 0).unwrap());
    }
}
