use chrono::{Datelike, Duration, NaiveDate, Weekday};

pub trait TradingDaysOffset {
    fn previous_trading_day(&self) -> NaiveDate;
}

impl TradingDaysOffset for NaiveDate {
    fn previous_trading_day(&self) -> NaiveDate {
        let offset = match self.weekday() {
            Weekday::Sun => 2,
            Weekday::Mon => 3,
            _ => 1,
        };

        *self - Duration::days(offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn previous_trading_day_saturday() {
        let tuesday = NaiveDate::from_ymd_opt(2024, 4, 20).unwrap();
        let expected = NaiveDate::from_ymd_opt(2024, 4, 19).unwrap();
        assert_eq!(expected, tuesday.previous_trading_day())
    }

    #[test]
    fn previous_trading_day_sunday() {
        let tuesday = NaiveDate::from_ymd_opt(2024, 4, 21).unwrap();
        let expected = NaiveDate::from_ymd_opt(2024, 4, 19).unwrap();
        assert_eq!(expected, tuesday.previous_trading_day())
    }

    #[test]
    fn previous_trading_day_monday() {
        let tuesday = NaiveDate::from_ymd_opt(2024, 4, 22).unwrap();
        let expected = NaiveDate::from_ymd_opt(2024, 4, 19).unwrap();
        assert_eq!(expected, tuesday.previous_trading_day())
    }

    #[test]
    fn previous_trading_day_tuesday() {
        let tuesday = NaiveDate::from_ymd_opt(2024, 4, 23).unwrap();
        let expected = NaiveDate::from_ymd_opt(2024, 4, 22).unwrap();
        assert_eq!(expected, tuesday.previous_trading_day())
    }
}