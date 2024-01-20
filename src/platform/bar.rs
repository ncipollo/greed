pub mod bar_request;
pub mod time_frame;

use chrono::{DateTime, Utc};
use num_decimal::Num;
use std::fmt::{Display, Formatter};
#[cfg(test)]
use crate::date::DateTimeFixture;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Bar {
    pub timestamp: DateTime<Utc>,
    /// The open price.
    pub open: Num,
    /// The close price.
    pub close: Num,
    /// The highest price.
    pub high: Num,
    /// The lowest price.
    pub low: Num,
    /// The trading volume.
    pub volume: usize,
}

impl Bar {
    pub fn average(&self) -> Num {
        (&self.low + &self.high) / 2
    }

    pub fn difference(&self) -> Num {
        &self.close - &self.open
    }

    pub fn difference_percent(&self) -> Num {
        (self.difference() / &self.open) * 100
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
    pub fn fixture(average: i64) -> Self {
        Self {
            timestamp: DateTimeFixture::utc(),
            open: Num::from(average - 100),
            close: Num::from(average + 100),
            low: Num::from(average - 100),
            high: Num::from(average + 100),
            volume: 100
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
    use num_decimal::Num;

    #[test]
    fn average() {
        let bar = Bar {
            low: Num::from(100),
            high: Num::from(200),
            ..Default::default()
        };
        assert_eq!(bar.average(), Num::from(150))
    }

    #[test]
    fn difference() {
        let bar = Bar {
            open: Num::from(200),
            close: Num::from(100),
            ..Default::default()
        };
        assert_eq!(bar.difference(), Num::from(-100))
    }

    #[test]
    fn difference_percent() {
        let bar = Bar {
            open: Num::from(200),
            close: Num::from(100),
            ..Default::default()
        };
        assert_eq!(bar.difference_percent(), Num::from(-50))
    }

    #[test]
    fn display() {
        let timestamp = date(1);
        let bar = Bar {
            timestamp,
            open: Num::from(1),
            close: Num::from(2),
            high: Num::from(4),
            low: Num::from(3),
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
            open: Num::from(100),
            close: Num::from(200),
            low: Num::from(0),
            high: Num::from(500),
            volume: 0,
        }
    }

    fn later_bar() -> Bar {
        Bar {
            timestamp: date(2),
            open: Num::from(300),
            close: Num::from(50),
            low: Num::from(200),
            high: Num::from(200),
            volume: 0,
        }
    }

    fn join_expected() -> Bar {
        Bar {
            timestamp: date(1),
            open: Num::from(100),
            close: Num::from(50),
            low: Num::from(0),
            high: Num::from(500),
            volume: 0,
        }
    }
}
