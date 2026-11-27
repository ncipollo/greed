use async_trait::async_trait;
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::sleep;

/// Where strategy and analysis code gets "now". Live runs use the system
/// clock; a backtest drives a `SimulatedClock` so every rule believes it is
/// the simulated moment.
pub trait Clock: Send + Sync {
    fn now(&self) -> DateTime<Utc>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

/// A clock that only moves when told to.
#[derive(Debug)]
pub struct SimulatedClock {
    now: Mutex<DateTime<Utc>>,
}

impl SimulatedClock {
    pub fn new(start: DateTime<Utc>) -> Self {
        Self {
            now: Mutex::new(start),
        }
    }

    pub fn set(&self, now: DateTime<Utc>) {
        *self.now.lock().unwrap() = now;
    }

    pub fn advance(&self, by: ChronoDuration) {
        *self.now.lock().unwrap() += by;
    }
}

impl Clock for SimulatedClock {
    fn now(&self) -> DateTime<Utc> {
        *self.now.lock().unwrap()
    }
}

/// How long to wait between tactics and ticks. Live runs actually sleep;
/// simulations skip the wait.
#[async_trait]
pub trait Pacer: Send + Sync {
    async fn pause(&self, duration: Duration);
}

#[derive(Debug, Default, Clone, Copy)]
pub struct TokioPacer;

#[async_trait]
impl Pacer for TokioPacer {
    async fn pause(&self, duration: Duration) {
        sleep(duration).await;
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct NoPacer;

#[async_trait]
impl Pacer for NoPacer {
    async fn pause(&self, _duration: Duration) {}
}

/// The clock and pacer a run is wired with. `Timing::default()` is live.
#[derive(Clone)]
pub struct Timing {
    pub clock: Arc<dyn Clock>,
    pub pacer: Arc<dyn Pacer>,
}

impl Timing {
    pub fn new(clock: Arc<dyn Clock>, pacer: Arc<dyn Pacer>) -> Self {
        Self { clock, pacer }
    }

    /// Simulated time with no waiting, for backtests.
    pub fn simulated(clock: Arc<SimulatedClock>) -> Self {
        Self::new(clock, Arc::new(NoPacer))
    }
}

impl Default for Timing {
    fn default() -> Self {
        Self::new(Arc::new(SystemClock), Arc::new(TokioPacer))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::TimeZone;

    fn at(day: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 1, day, 12, 0, 0).unwrap()
    }

    #[test]
    fn simulated_clock_holds_still_until_moved() {
        let clock = SimulatedClock::new(at(1));

        assert_eq!(clock.now(), at(1));
        clock.advance(ChronoDuration::days(2));
        assert_eq!(clock.now(), at(3));
        clock.set(at(10));
        assert_eq!(clock.now(), at(10));
    }

    #[test]
    fn system_clock_is_close_to_now() {
        let drift = (SystemClock.now() - Utc::now()).num_seconds().abs();

        assert!(drift < 5);
    }

    #[tokio::test(start_paused = true)]
    async fn tokio_pacer_waits_and_no_pacer_does_not() {
        let start = tokio::time::Instant::now();

        NoPacer.pause(Duration::from_secs(60)).await;
        assert_eq!(start.elapsed(), Duration::ZERO);

        TokioPacer.pause(Duration::from_secs(60)).await;
        assert!(start.elapsed() >= Duration::from_secs(60));
    }
}
