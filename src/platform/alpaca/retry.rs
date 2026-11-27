use std::future::Future;
use std::time::Duration;
use tokio::time::sleep;

const MAX_ATTEMPTS: u32 = 6;
const INITIAL_BACKOFF: Duration = Duration::from_secs(1);

/// Runs `request`, retrying with exponential backoff while `is_rate_limited`
/// says the error is a rate-limit rejection (HTTP 429). Any other error, or
/// running out of attempts, is returned to the caller.
///
/// apca doesn't expose the response's `Retry-After` header, so the delay is a
/// plain 1s, 2s, 4s, ... backoff.
pub(super) async fn on_rate_limit<T, E, F, Fut>(
    is_rate_limited: impl Fn(&E) -> bool,
    mut request: F,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
{
    let mut attempt = 0;
    loop {
        match request().await {
            Err(e) if attempt + 1 < MAX_ATTEMPTS && is_rate_limited(&e) => {
                sleep(INITIAL_BACKOFF * 2u32.pow(attempt)).await;
                attempt += 1;
            }
            result => return result,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use tokio::time::Instant;

    #[derive(Debug, PartialEq)]
    enum TestError {
        RateLimited,
        Other,
    }

    fn is_rate_limited(error: &TestError) -> bool {
        *error == TestError::RateLimited
    }

    #[tokio::test(start_paused = true)]
    async fn returns_immediately_on_success() {
        let calls = AtomicU32::new(0);

        let result = on_rate_limit(is_rate_limited, || async {
            calls.fetch_add(1, Ordering::SeqCst);
            Ok::<_, TestError>(7)
        })
        .await;

        assert_eq!(result, Ok(7));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test(start_paused = true)]
    async fn retries_rate_limited_requests_with_growing_backoff() {
        let calls = AtomicU32::new(0);
        let start = Instant::now();

        let result = on_rate_limit(is_rate_limited, || async {
            if calls.fetch_add(1, Ordering::SeqCst) < 3 {
                Err(TestError::RateLimited)
            } else {
                Ok(())
            }
        })
        .await;

        assert_eq!(result, Ok(()));
        assert_eq!(calls.load(Ordering::SeqCst), 4);
        // 1s + 2s + 4s of backoff before the fourth attempt succeeds.
        assert_eq!(start.elapsed(), Duration::from_secs(7));
    }

    #[tokio::test(start_paused = true)]
    async fn gives_up_after_the_maximum_attempts() {
        let calls = AtomicU32::new(0);

        let result = on_rate_limit(is_rate_limited, || async {
            calls.fetch_add(1, Ordering::SeqCst);
            Err::<(), _>(TestError::RateLimited)
        })
        .await;

        assert_eq!(result, Err(TestError::RateLimited));
        assert_eq!(calls.load(Ordering::SeqCst), MAX_ATTEMPTS);
    }

    #[tokio::test(start_paused = true)]
    async fn does_not_retry_other_errors() {
        let calls = AtomicU32::new(0);

        let result = on_rate_limit(is_rate_limited, || async {
            calls.fetch_add(1, Ordering::SeqCst);
            Err::<(), _>(TestError::Other)
        })
        .await;

        assert_eq!(result, Err(TestError::Other));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
}
