pub mod bar_request;
pub mod time_frame;

#[cfg(test)]
use crate::date::DateTimeFixture;
use chrono::{DateTime, Utc};
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Bar {
    pub timestamp: DateTime<Utc>,
    /// The open price.
    pub open: f64,
    /// The close price.
    pub close: f64,
    /// The highest price.
    pub high: f64,
    /// The lowest price.
    pub low: f64,
    /// The trading volume.
    pub volume: usize,
}

impl Bar {
    pub fn average(&self) -> f64 {
        (&self.low + &self.high) / 2.0
    }

    pub fn difference(&self) -> f64 {
        &self.close - &self.open
    }

    pub fn difference_percent(&self) -> f64 {
        (self.difference() / &self.open) * 100.0
    }

    pub fn join(&self, other: &Bar) -> Bar {
        let (timestamp, open, close) = if self.timestamp < other.timestamp {
            (self.timestamp, &self.open, &other.close)
        } else {
            (other.timestamp, &other.open, &self.close)
        };

        Self {
            timestamp,
            open: open.clone(),
            close: close.clone(),
            low: self.low.clone().min(other.clone().low),
            high: self.high.clone().max(other.clone().high),
            ..Default::default()
        }
    }

    #[cfg(test)]
    pub fn fixture(average: f64) -> Self {
        Self {
            timestamp: DateTimeFixture::utc(),
            open: average - 100.0,
            close: average + 100.0,
            low: average - 100.0,
            high: average + 100.0,
            volume: 100,
        }
    }
}

impl Display for Bar {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let formatted_time = self.timestamp.format("%m/%d/%y %H:%M:%S %Z");
        write!(
            f,
            "{} - open: {}, close: {}, high: {}, low: {}",
            formatted_time, self.open, self.close, self.high, self.low
        )
    }
}

#[cfg(test)]
mod test {
    use crate::platform::bar::Bar;
    use chrono::{DateTime, TimeZone, Utc};

    #[test]
    fn average() {
        let bar = Bar {
            low: 100.0,
            high: 200.0,
            ..Default::default()
        };
        assert_eq!(bar.average(), 150.0)
    }

    #[test]
    fn difference() {
        let bar = Bar {
            open: 200.0,
            close: 100.0,
            ..Default::default()
        };
        assert_eq!(bar.difference(), -100.0)
    }

    #[test]
    fn difference_percent() {
        let bar = Bar {
            open: 200.0,
            close: 100.0,
            ..Default::default()
        };
        assert_eq!(bar.difference_percent(), -50.0)
    }

    #[test]
    fn display() {
        let timestamp = date(1);
        let bar = Bar {
            timestamp,
            open: 1.0,
            close: 2.0,
            high: 4.0,
            low: 3.0,
            ..Default::default()
        };
        let expected = "11/01/23 13:00:00 UTC - open: 1, close: 2, high: 4, low: 3";
        assert_eq!(bar.to_string(), expected)
    }

    #[test]
    fn join_earlier_first() {
        let joined = earlier_bar().join(&later_bar());
        assert_eq!(joined, join_expected())
    }

    #[test]
    fn join_later_first() {
        let joined = later_bar().join(&earlier_bar());
        assert_eq!(joined, join_expected())
    }

    fn date(day: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2023, 11, day, 13, 0, 0)
            .earliest()
            .expect("failed to create test date")
    }

    fn earlier_bar() -> Bar {
        Bar {
            timestamp: date(1),
            open: 100.0,
            close: 200.0,
            low: 0.0,
            high: 500.0,
            volume: 0,
        }
    }

    fn later_bar() -> Bar {
        Bar {
            timestamp: date(2),
            open: 300.0,
            close: 50.0,
            low: 200.0,
            high: 200.0,
            volume: 0,
        }
    }

    fn join_expected() -> Bar {
        Bar {
            timestamp: date(1),
            open: 100.0,
            close: 50.0,
            low: 0.0,
            high: 500.0,
            volume: 0,
        }
    }
}
