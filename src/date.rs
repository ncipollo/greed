use crate::error::GreedError;
use chrono::{DateTime, NaiveDate, NaiveDateTime, TimeZone, Utc};
use std::ops::Range;

const DATE_FORMAT: &str = "%Y-%m-%d";
const DATE_TIME_FORMATS: [&str; 4] = [
    "%Y-%m-%d %H:%M",
    "%Y-%m-%d %H:%M:%S",
    "%Y-%m-%dT%H:%M",
    "%Y-%m-%dT%H:%M:%S",
];

/// A user-supplied date or date-time (interpreted as UTC). `date_only` records
/// that no time was given, so callers can decide whether the date means the
/// start of that day or the whole day.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParsedMoment {
    pub at: DateTime<Utc>,
    pub date_only: bool,
}

/// Parses `YYYY-MM-DD` (as midnight UTC) or `YYYY-MM-DD HH:MM[:SS]` /
/// `YYYY-MM-DDTHH:MM[:SS]`.
pub fn parse_moment(value: &str) -> Result<ParsedMoment, GreedError> {
    for format in DATE_TIME_FORMATS {
        if let Ok(naive) = NaiveDateTime::parse_from_str(value, format) {
            return Ok(ParsedMoment {
                at: naive.to_utc(),
                date_only: false,
            });
        }
    }
    if let Ok(date) = NaiveDate::parse_from_str(value, DATE_FORMAT) {
        let naive = date.and_hms_opt(0, 0, 0).expect("midnight is always valid");
        return Ok(ParsedMoment {
            at: naive.to_utc(),
            date_only: true,
        });
    }
    Err(GreedError::new(&format!(
        "unable to parse '{value}' as a date/time; expected 'YYYY-MM-DD', \
         'YYYY-MM-DD HH:MM[:SS]' or 'YYYY-MM-DDTHH:MM[:SS]'"
    )))
}

pub trait NaiveDateTimeConvert {
    type ToType;
    fn to_utc(&self) -> Self::ToType;
}

impl NaiveDateTimeConvert for NaiveDateTime {
    type ToType = DateTime<Utc>;

    fn to_utc(&self) -> DateTime<Utc> {
        Utc.from_utc_datetime(self)
    }
}

impl NaiveDateTimeConvert for Range<NaiveDateTime> {
    type ToType = Range<DateTime<Utc>>;

    fn to_utc(&self) -> Range<DateTime<Utc>> {
        self.start.to_utc()..self.end.to_utc()
    }
}

#[cfg(test)]
use chrono::Local;

#[cfg(test)]
pub struct DateTimeFixture {}

#[cfg(test)]
impl DateTimeFixture {
    pub fn local() -> DateTime<Local> {
        Local
            .with_ymd_and_hms(2023, 12, 4, 8, 0, 0)
            .earliest()
            .expect("failed to get local date")
    }

    pub fn utc() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2023, 12, 4, 8, 0, 0)
            .earliest()
            .expect("failed to get utc date")
    }

    pub fn naive_utc() -> NaiveDateTime {
        Self::utc().naive_utc()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    fn utc(y: i32, mo: u32, d: u32, h: u32, mi: u32, s: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(y, mo, d, h, mi, s).unwrap()
    }

    #[test]
    fn parse_moment_date_only_is_midnight_and_flagged() {
        let parsed = parse_moment("2026-10-01").expect("should parse");
        assert_eq!(
            parsed,
            ParsedMoment {
                at: utc(2026, 10, 1, 0, 0, 0),
                date_only: true
            }
        );
    }

    #[test]
    fn parse_moment_accepts_every_date_time_format() {
        let cases = [
            ("2026-10-01 08:15", utc(2026, 10, 1, 8, 15, 0)),
            ("2026-10-01 08:15:30", utc(2026, 10, 1, 8, 15, 30)),
            ("2026-10-01T08:15", utc(2026, 10, 1, 8, 15, 0)),
            ("2026-10-01T08:15:30", utc(2026, 10, 1, 8, 15, 30)),
        ];
        for (input, expected) in cases {
            let parsed = parse_moment(input).expect(input);
            assert_eq!(parsed.at, expected, "{input}");
            assert!(!parsed.date_only, "{input}");
        }
    }

    #[test]
    fn parse_moment_rejects_garbage() {
        assert!(parse_moment("garbage").is_err());
        assert!(parse_moment("2026-13-01").is_err());
        assert!(parse_moment("").is_err());
    }

    #[test]
    fn naive_date_time_to_utc() {
        let naive = DateTimeFixture::naive_utc();
        assert_eq!(DateTimeFixture::utc(), naive.to_utc())
    }

    #[test]
    fn naive_range_to_utc() {
        let start = DateTimeFixture::naive_utc();
        let end = start.with_day(start.day()).expect("failed to set day");

        let expected_start = DateTimeFixture::utc();
        let expected_end = end.to_utc();
        assert_eq!(expected_start..expected_end, (start..end).to_utc())
    }
}
