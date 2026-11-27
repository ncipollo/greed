use crate::platform::rate_limit::RateLimit;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::{sleep_until, Instant};

/// Target this share of the platform's limit, leaving headroom for clock skew
/// and for requests made by other clients on the same account.
const SAFETY_MARGIN: f64 = 0.9;

/// Spaces calls out evenly so a caller stays under a [`RateLimit`].
///
/// Each `acquire` reserves the next free slot and sleeps until it arrives, so
/// concurrent callers queue up in order rather than bursting together.
pub struct RateLimiter {
    spacing: Duration,
    next_slot: Mutex<Instant>,
}

impl RateLimiter {
    pub fn new(limit: RateLimit) -> Self {
        let targeted_requests = f64::from(limit.requests()) * SAFETY_MARGIN;
        Self {
            spacing: limit.per().div_f64(targeted_requests),
            next_slot: Mutex::new(Instant::now()),
        }
    }

    #[cfg(test)]
    pub fn spacing(&self) -> Duration {
        self.spacing
    }

    /// Waits until the caller may make one request.
    pub async fn acquire(&self) {
        let slot = {
            let mut next_slot = self.next_slot.lock().await;
            let slot = (*next_slot).max(Instant::now());
            *next_slot = slot + self.spacing;
            slot
        };
        sleep_until(slot).await;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// Tokio's paused clock rounds sleeps up to the next millisecond, so compare with a little slack.
    fn assert_close(actual: Duration, expected: Duration) {
        let diff = actual.abs_diff(expected);
        assert!(
            diff <= Duration::from_millis(2),
            "expected about {expected:?}, got {actual:?}"
        );
    }

    fn limiter(requests_per_minute: u32) -> RateLimiter {
        RateLimiter::new(RateLimit::per_minute(requests_per_minute).expect("valid limit"))
    }

    #[test]
    fn spacing_targets_ninety_percent_of_the_limit() {
        // 200/min * 0.9 = 180/min, so one request every 1/3 second.
        let spacing = limiter(200).spacing();
        assert_eq!(spacing.as_millis(), 333);
    }

    #[tokio::test(start_paused = true)]
    async fn first_acquire_is_immediate() {
        let limiter = limiter(60);
        let start = Instant::now();

        limiter.acquire().await;

        assert_eq!(start.elapsed(), Duration::ZERO);
    }

    #[tokio::test(start_paused = true)]
    async fn back_to_back_acquires_are_spaced_apart() {
        let limiter = limiter(60);
        let start = Instant::now();

        for _ in 0..4 {
            limiter.acquire().await;
        }

        // The first is free, so 3 intervals elapse across 4 acquires.
        assert_close(start.elapsed(), limiter.spacing() * 3);
    }

    #[tokio::test(start_paused = true)]
    async fn idle_time_does_not_build_up_a_burst() {
        let limiter = limiter(60);
        limiter.acquire().await;
        tokio::time::advance(limiter.spacing() * 10).await;

        let start = Instant::now();
        limiter.acquire().await;
        limiter.acquire().await;

        assert_close(start.elapsed(), limiter.spacing());
    }

    #[tokio::test(start_paused = true)]
    async fn concurrent_callers_are_queued_in_order() {
        let limiter = std::sync::Arc::new(limiter(60));
        let start = Instant::now();

        let tasks: Vec<_> = (0..3)
            .map(|_| {
                let limiter = limiter.clone();
                tokio::spawn(async move {
                    limiter.acquire().await;
                    start.elapsed()
                })
            })
            .collect();

        let mut finished = Vec::new();
        for task in tasks {
            finished.push(task.await.expect("task should not panic"));
        }
        finished.sort();
        let spacing = limiter.spacing();
        assert_eq!(finished[0], Duration::ZERO);
        assert_close(finished[1], spacing);
        assert_close(finished[2], spacing * 2);
    }
}
