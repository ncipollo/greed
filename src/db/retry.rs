use crate::error::GreedError;
use duckdb::Connection;
use std::time::Duration;

const MAX_ATTEMPTS: u32 = 7;
const INITIAL_BACKOFF_MS: u64 = 40;

pub(crate) fn is_lock_conflict(message: &str) -> bool {
    message.contains("Could not set lock on file")
}

pub(crate) async fn with_backoff<F>(open: F) -> Result<Connection, GreedError>
where
    F: Fn() -> Result<Connection, GreedError>,
{
    let mut attempt = 0;
    loop {
        match open() {
            Ok(connection) => return Ok(connection),
            Err(e) if attempt + 1 < MAX_ATTEMPTS && is_lock_conflict(&e.to_string()) => {
                let backoff_ms = INITIAL_BACKOFF_MS * 2u64.pow(attempt);
                tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
                attempt += 1;
            }
            Err(e) => return Err(e),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::error::GreedError;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[test]
    fn is_lock_conflict_matches_duckdb_lock_message() {
        assert!(is_lock_conflict(
            "IO Error: Could not set lock on file \"greed.duckdb\": Conflicting lock is held"
        ));
    }

    #[test]
    fn is_lock_conflict_rejects_unrelated_message() {
        assert!(!is_lock_conflict("IO Error: file not found"));
    }

    #[tokio::test]
    async fn with_backoff_retries_until_success() {
        let attempts = AtomicU32::new(0);
        let result = with_backoff(|| {
            let attempt = attempts.fetch_add(1, Ordering::SeqCst);
            if attempt < 2 {
                Err(GreedError::new(
                    "IO Error: Could not set lock on file \"test.duckdb\"",
                ))
            } else {
                Connection::open_in_memory().map_err(GreedError::from)
            }
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn with_backoff_does_not_retry_non_lock_errors() {
        let attempts = AtomicU32::new(0);
        let result = with_backoff(|| {
            attempts.fetch_add(1, Ordering::SeqCst);
            Err(GreedError::new("IO Error: file not found"))
        })
        .await;

        assert!(result.is_err());
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn with_backoff_gives_up_after_max_attempts() {
        let attempts = AtomicU32::new(0);
        let result = with_backoff(|| {
            attempts.fetch_add(1, Ordering::SeqCst);
            Err(GreedError::new(
                "IO Error: Could not set lock on file \"test.duckdb\"",
            ))
        })
        .await;

        assert!(result.is_err());
        assert_eq!(attempts.load(Ordering::SeqCst), MAX_ATTEMPTS);
    }
}
