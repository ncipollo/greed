use crate::error::GreedError;
use std::time::Duration;

/// How many requests a platform allows within a window of time.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RateLimit {
    requests: u32,
    per: Duration,
}

impl RateLimit {
    pub fn new(requests: u32, per: Duration) -> Result<Self, GreedError> {
        if requests == 0 {
            return Err(GreedError::new("rate limit must allow at least 1 request"));
        }
        if per.is_zero() {
            return Err(GreedError::new("rate limit window must be longer than 0"));
        }
        Ok(Self { requests, per })
    }

    pub fn per_minute(requests: u32) -> Result<Self, GreedError> {
        Self::new(requests, Duration::from_secs(60))
    }

    pub fn requests(&self) -> u32 {
        self.requests
    }

    pub fn per(&self) -> Duration {
        self.per
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn per_minute_uses_a_sixty_second_window() {
        let limit = RateLimit::per_minute(200).expect("valid limit");
        assert_eq!(limit.requests(), 200);
        assert_eq!(limit.per(), Duration::from_secs(60));
    }

    #[test]
    fn zero_requests_is_rejected() {
        assert!(RateLimit::per_minute(0).is_err());
    }

    #[test]
    fn zero_length_window_is_rejected() {
        assert!(RateLimit::new(10, Duration::ZERO).is_err());
    }
}
