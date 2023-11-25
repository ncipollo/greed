pub mod bar_request;
pub mod time_frame;

use crate::asset::AssetSymbol;
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

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Bars {
    pub symbol: AssetSymbol,
    pub bars: Vec<Bar>,
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
