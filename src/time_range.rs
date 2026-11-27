use crate::date::parse_moment;
use crate::error::GreedError;
use chrono::{DateTime, Duration, Months, Utc};
use std::ops::Range;

/// A user's request for a window of time, as given on the command line.
///
/// Either explicit (`from`, optionally `to`) or relative to now (`years`,
/// `months`, `days`, which may be combined). With nothing set the window is
/// the last year. All times are UTC.
#[derive(Debug, Default, PartialEq)]
pub struct TimeRangeSpec {
    pub from: Option<String>,
    pub to: Option<String>,
    pub days: Option<u32>,
    pub months: Option<u32>,
    pub years: Option<u32>,
}

impl TimeRangeSpec {
    pub fn resolve(&self, now: DateTime<Utc>) -> Result<Range<DateTime<Utc>>, GreedError> {
        let has_relative = self.days.is_some() || self.months.is_some() || self.years.is_some();
        if has_relative && (self.from.is_some() || self.to.is_some()) {
            return Err(GreedError::new(
                "a relative range (days/months/years) can't be combined with from/to",
            ));
        }
        let range = if has_relative {
            self.resolve_relative(now)?
        } else if self.from.is_some() || self.to.is_some() {
            self.resolve_explicit(now)?
        } else {
            Self::months_back(now, 12)?..now
        };
        if range.start >= range.end {
            return Err(GreedError::new(&format!(
                "time range start ({}) must be before its end ({})",
                range.start, range.end
            )));
        }
        Ok(range)
    }

    fn resolve_relative(&self, now: DateTime<Utc>) -> Result<Range<DateTime<Utc>>, GreedError> {
        let total_months = self
            .years
            .unwrap_or(0)
            .checked_mul(12)
            .and_then(|months| months.checked_add(self.months.unwrap_or(0)))
            .ok_or_else(|| GreedError::new("relative time range is too large"))?;
        let days = self.days.unwrap_or(0);
        if total_months == 0 && days == 0 {
            return Err(GreedError::new(
                "relative time range must be at least 1 day",
            ));
        }
        let start = Self::months_back(now, total_months)?
            .checked_sub_signed(Duration::days(i64::from(days)))
            .ok_or_else(|| GreedError::new("relative time range is too large"))?;
        Ok(start..now)
    }

    fn resolve_explicit(&self, now: DateTime<Utc>) -> Result<Range<DateTime<Utc>>, GreedError> {
        let Some(from) = &self.from else {
            return Err(GreedError::new("'to' requires 'from'"));
        };
        let start = parse_moment(from)?.at;
        let end = match &self.to {
            Some(to) => {
                let moment = parse_moment(to)?;
                // A bare date means the whole day, so the range ends at the next midnight.
                let end = if moment.date_only {
                    moment.at + Duration::days(1)
                } else {
                    moment.at
                };
                end.min(now)
            }
            None => now,
        };
        Ok(start..end)
    }

    fn months_back(now: DateTime<Utc>, months: u32) -> Result<DateTime<Utc>, GreedError> {
        now.checked_sub_months(Months::new(months))
            .ok_or_else(|| GreedError::new("relative time range is too large"))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::TimeZone;

    fn utc(y: i32, mo: u32, d: u32, h: u32, mi: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(y, mo, d, h, mi, 0).unwrap()
    }

    fn now() -> DateTime<Utc> {
        utc(2026, 10, 17, 12, 0)
    }

    fn explicit(from: Option<&str>, to: Option<&str>) -> TimeRangeSpec {
        TimeRangeSpec {
            from: from.map(str::to_string),
            to: to.map(str::to_string),
            ..Default::default()
        }
    }

    #[test]
    fn resolve_defaults_to_last_year() {
        let range = TimeRangeSpec::default().resolve(now()).expect("resolve");
        assert_eq!(range, utc(2025, 10, 17, 12, 0)..now());
    }

    #[test]
    fn resolve_days() {
        let spec = TimeRangeSpec {
            days: Some(5),
            ..Default::default()
        };
        assert_eq!(
            spec.resolve(now()).unwrap(),
            utc(2026, 10, 12, 12, 0)..now()
        );
    }

    #[test]
    fn resolve_months() {
        let spec = TimeRangeSpec {
            months: Some(3),
            ..Default::default()
        };
        assert_eq!(spec.resolve(now()).unwrap(), utc(2026, 7, 17, 12, 0)..now());
    }

    #[test]
    fn resolve_years() {
        let spec = TimeRangeSpec {
            years: Some(2),
            ..Default::default()
        };
        assert_eq!(
            spec.resolve(now()).unwrap(),
            utc(2024, 10, 17, 12, 0)..now()
        );
    }

    #[test]
    fn resolve_combines_years_months_and_days() {
        let spec = TimeRangeSpec {
            years: Some(1),
            months: Some(6),
            days: Some(2),
            ..Default::default()
        };
        assert_eq!(spec.resolve(now()).unwrap(), utc(2025, 4, 15, 12, 0)..now());
    }

    #[test]
    fn resolve_month_end_clamps_to_shorter_month() {
        let end = utc(2026, 3, 31, 0, 0);
        let spec = TimeRangeSpec {
            months: Some(1),
            ..Default::default()
        };
        assert_eq!(spec.resolve(end).unwrap().start, utc(2026, 2, 28, 0, 0));
    }

    #[test]
    fn resolve_leap_day_minus_a_year_clamps() {
        let end = utc(2028, 2, 29, 0, 0);
        let spec = TimeRangeSpec {
            years: Some(1),
            ..Default::default()
        };
        assert_eq!(spec.resolve(end).unwrap().start, utc(2027, 2, 28, 0, 0));
    }

    #[test]
    fn resolve_zero_relative_span_is_an_error() {
        let spec = TimeRangeSpec {
            days: Some(0),
            ..Default::default()
        };
        assert!(spec.resolve(now()).is_err());
    }

    #[test]
    fn resolve_huge_relative_span_is_an_error_not_a_panic() {
        let spec = TimeRangeSpec {
            years: Some(u32::MAX),
            ..Default::default()
        };
        assert!(spec.resolve(now()).is_err());
    }

    #[test]
    fn resolve_from_only_ends_at_now() {
        let range = explicit(Some("2026-10-01"), None).resolve(now()).unwrap();
        assert_eq!(range, utc(2026, 10, 1, 0, 0)..now());
    }

    #[test]
    fn resolve_from_and_to_with_times_uses_them_as_given() {
        let spec = explicit(Some("2026-10-01 09:30"), Some("2026-10-02 14:30"));
        assert_eq!(
            spec.resolve(now()).unwrap(),
            utc(2026, 10, 1, 9, 30)..utc(2026, 10, 2, 14, 30)
        );
    }

    #[test]
    fn resolve_date_only_to_includes_the_whole_day() {
        let spec = explicit(Some("2026-10-01"), Some("2026-10-02"));
        assert_eq!(
            spec.resolve(now()).unwrap(),
            utc(2026, 10, 1, 0, 0)..utc(2026, 10, 3, 0, 0)
        );
    }

    #[test]
    fn resolve_future_to_is_clamped_to_now() {
        let spec = explicit(Some("2026-10-01"), Some("2027-01-01"));
        assert_eq!(spec.resolve(now()).unwrap().end, now());
    }

    #[test]
    fn resolve_date_only_to_today_is_clamped_to_now() {
        let spec = explicit(Some("2026-10-01"), Some("2026-10-17"));
        assert_eq!(spec.resolve(now()).unwrap().end, now());
    }

    #[test]
    fn resolve_from_in_the_future_is_an_error() {
        assert!(explicit(Some("2027-01-01"), None).resolve(now()).is_err());
    }

    #[test]
    fn resolve_to_before_from_is_an_error() {
        let spec = explicit(Some("2026-10-05"), Some("2026-10-01 00:00"));
        assert!(spec.resolve(now()).is_err());
    }

    #[test]
    fn resolve_to_without_from_is_an_error() {
        assert!(explicit(None, Some("2026-10-01")).resolve(now()).is_err());
    }

    #[test]
    fn resolve_from_with_relative_is_an_error() {
        let spec = TimeRangeSpec {
            from: Some("2026-10-01".to_string()),
            days: Some(3),
            ..Default::default()
        };
        assert!(spec.resolve(now()).is_err());
    }

    #[test]
    fn resolve_rejects_unparseable_dates() {
        assert!(explicit(Some("nope"), None).resolve(now()).is_err());
        assert!(explicit(Some("2026-10-01"), Some("nope"))
            .resolve(now())
            .is_err());
    }
}
