use crate::date::NaiveDateTimeConvert;
use chrono::{DateTime, Duration, NaiveDate, Utc};
use std::ops::Range;
use crate::trading_days::TradingDaysOffset;

pub struct FetcherTimeRanges {
    now: DateTime<Utc>,
}

impl FetcherTimeRanges {
    pub fn new() -> Self {
        Self { now: Utc::now() }
    }

    /// Yesterday from 00:00:00 to 23:59:00
    pub fn last_trading_day_range(&self) -> Range<DateTime<Utc>> {
        let now_date = self.now.date_naive();
        let last_trading_date = now_date.previous_trading_day();
        Self::create_date_time_range(last_trading_date, last_trading_date)
    }

    /// X days in the past until yesterday at 23:59:00
    pub fn last_x_days(&self, x_days: i64) -> Range<DateTime<Utc>> {
        let now_date = self.now.date_naive();
        let start_date = now_date - Duration::days(x_days);
        let yesterday_date = now_date - Duration::days(1);
        Self::create_date_time_range(start_date, yesterday_date)
    }

    fn create_date_time_range(start: NaiveDate, end: NaiveDate) -> Range<DateTime<Utc>> {
        let adjusted_start = start.and_hms_opt(0, 0, 0).expect("start was invalid");
        let adjusted_end = end.and_hms_opt(23, 59, 0).expect("end was invalid");
        (adjusted_start..adjusted_end).to_utc()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::date::DateTimeFixture;
    use chrono::TimeZone;

    #[test]
    fn yesterday_range() {
        let ranges = create_ranges();
        let expected_start = Utc
            .with_ymd_and_hms(2023, 12, 01, 0, 0, 0)
            .earliest()
            .unwrap();
        let expected_end = Utc
            .with_ymd_and_hms(2023, 12, 01, 23, 59, 0)
            .earliest()
            .unwrap();
        let expected = expected_start..expected_end;
        assert_eq!(expected, ranges.last_trading_day_range())
    }

    #[test]
    fn last_x_days() {
        let ranges = create_ranges();
        let expected_start = Utc
            .with_ymd_and_hms(2023, 11, 24, 0, 0, 0)
            .earliest()
            .unwrap();
        let expected_end = Utc
            .with_ymd_and_hms(2023, 12, 03, 23, 59, 0)
            .earliest()
            .unwrap();
        let expected = expected_start..expected_end;
        assert_eq!(expected, ranges.last_x_days(10))
    }

    fn create_ranges() -> FetcherTimeRanges {
        FetcherTimeRanges {
            now: DateTimeFixture::utc(),
        }
    }
}
