use crate::date::NaiveDateTimeConvert;
use crate::error::GreedError;
use chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, Utc};
use std::ops::Range;

#[derive(Debug, Default, PartialEq)]
pub struct HistoryRange {
    pub from: Option<String>,
    pub to: Option<String>,
    pub last_hour: bool,
    pub hours: Option<i64>,
    pub days: Option<i64>,
}

impl HistoryRange {
    pub fn resolve(&self, now: DateTime<Utc>) -> Result<Range<DateTime<Utc>>, GreedError> {
        if self.last_hour {
            return Ok((now - Duration::hours(1))..now);
        }
        if let Some(hours) = self.hours {
            return Ok((now - Duration::hours(hours))..now);
        }
        if let Some(days) = self.days {
            return Ok((now - Duration::days(days))..now);
        }
        if let Some(from) = &self.from {
            let start = parse_date_time(from)?;
            let end = match &self.to {
                Some(to) => parse_date_time(to)?,
                None => now,
            };
            return Ok(start..end);
        }
        Ok((now - Duration::days(1))..now)
    }
}

fn parse_date_time(value: &str) -> Result<DateTime<Utc>, GreedError> {
    if let Ok(naive) = NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M") {
        return Ok(naive.to_utc());
    }
    if let Ok(date) = NaiveDate::parse_from_str(value, "%Y-%m-%d") {
        let naive = date.and_hms_opt(0, 0, 0).expect("midnight is always valid");
        return Ok(naive.to_utc());
    }
    Err(GreedError::new(&format!(
        "unable to parse '{value}' as a date/time; expected format '%Y-%m-%d %H:%M' or '%Y-%m-%d'"
    )))
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::TimeZone;

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 10, 17, 12, 0, 0).unwrap()
    }

    fn range_args() -> HistoryRange {
        HistoryRange::default()
    }

    #[test]
    fn resolve_defaults_to_last_day() {
        let range = range_args().resolve(now()).expect("resolve should succeed");
        assert_eq!(range, (now() - Duration::days(1))..now());
    }

    #[test]
    fn resolve_last_hour() {
        let args = HistoryRange {
            last_hour: true,
            ..range_args()
        };
        let range = args.resolve(now()).expect("resolve should succeed");
        assert_eq!(range, (now() - Duration::hours(1))..now());
    }

    #[test]
    fn resolve_hours() {
        let args = HistoryRange {
            hours: Some(3),
            ..range_args()
        };
        let range = args.resolve(now()).expect("resolve should succeed");
        assert_eq!(range, (now() - Duration::hours(3))..now());
    }

    #[test]
    fn resolve_days() {
        let args = HistoryRange {
            days: Some(5),
            ..range_args()
        };
        let range = args.resolve(now()).expect("resolve should succeed");
        assert_eq!(range, (now() - Duration::days(5))..now());
    }

    #[test]
    fn resolve_explicit_from_only_defaults_end_to_now() {
        let args = HistoryRange {
            from: Some("2026-10-01".to_string()),
            ..range_args()
        };
        let range = args.resolve(now()).expect("resolve should succeed");
        let expected_start = Utc.with_ymd_and_hms(2026, 10, 1, 0, 0, 0).unwrap();
        assert_eq!(range, expected_start..now());
    }

    #[test]
    fn resolve_explicit_from_and_to() {
        let args = HistoryRange {
            from: Some("2026-10-01".to_string()),
            to: Some("2026-10-02 14:30".to_string()),
            ..range_args()
        };
        let range = args.resolve(now()).expect("resolve should succeed");
        let expected_start = Utc.with_ymd_and_hms(2026, 10, 1, 0, 0, 0).unwrap();
        let expected_end = Utc.with_ymd_and_hms(2026, 10, 2, 14, 30, 0).unwrap();
        assert_eq!(range, expected_start..expected_end);
    }

    #[test]
    fn resolve_rejects_unparseable_from() {
        let args = HistoryRange {
            from: Some("not-a-date".to_string()),
            ..range_args()
        };
        assert!(args.resolve(now()).is_err());
    }

    #[test]
    fn parse_date_time_accepts_date_and_time() {
        let parsed = parse_date_time("2026-10-01 08:15").expect("should parse");
        assert_eq!(parsed, Utc.with_ymd_and_hms(2026, 10, 1, 8, 15, 0).unwrap());
    }

    #[test]
    fn parse_date_time_accepts_date_only_as_midnight() {
        let parsed = parse_date_time("2026-10-01").expect("should parse");
        assert_eq!(parsed, Utc.with_ymd_and_hms(2026, 10, 1, 0, 0, 0).unwrap());
    }

    #[test]
    fn parse_date_time_rejects_garbage() {
        assert!(parse_date_time("garbage").is_err());
    }
}
