use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use std::ops::Range;

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
            .with_ymd_and_hms(2023, 12, 01, 8, 0, 0)
            .earliest()
            .expect("failed to get local date")
    }

    pub fn utc() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2023, 12, 01, 8, 0, 0)
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
