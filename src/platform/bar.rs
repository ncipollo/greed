pub mod bar_request;
pub mod time_frame;

use chrono::{DateTime, Utc};
use num_decimal::Num;
use std::fmt::{Display, Formatter};

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
        let timestamp = date();
        let bar = Bar {
            timestamp,
            open: Num::from(1),
            close: Num::from(2),
            high: Num::from(4),
            low: Num::from(3),
            ..Default::default()
        };
        let expected = "11/25/23 13:00:00 UTC - open: 1, close: 2, high: 4, low: 3";
        assert_eq!(bar.to_string(), expected)
    }

    fn date() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2023, 11, 25, 13, 0, 0)
            .earliest()
            .expect("failed to create test date")
    }
}
