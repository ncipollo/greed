use crate::asset::AssetSymbol;
use crate::platform::bar::Bar;
use itertools::Itertools;
use num_decimal::Num;
use std::borrow::Cow;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Bars {
    pub symbol: AssetSymbol,
    pub bars: Vec<Bar>,
}

impl Bars {
    pub fn average_median(&self) -> Option<Num> {
        self.median(|b| Cow::Owned(b.average()))
    }

    pub fn close_median(&self) -> Option<Num> {
        self.median(|b| Cow::Borrowed(&b.close))
    }

    fn median<F>(&self, func: F) -> Option<Num>
    where
        F: FnMut(&Bar) -> Cow<Num>,
    {
        if self.is_empty() {
            return None;
        }
        let sorted_bars = self.bars.iter().map(func).sorted().collect::<Vec<_>>();
        let middle = sorted_bars.len() / 2;
        return Some(sorted_bars[middle].clone().into_owned());
    }

    pub fn period_bar(&self) -> Option<Bar> {
        let first_bar = self.bars.first()?;
        let last_bar = self.bars.last()?;
        let joined = first_bar.join(last_bar);
        Some(joined)
    }

    pub fn is_empty(&self) -> bool {
        self.bars.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, TimeZone, Utc};

    #[test]
    fn average_median() {
        let bar_vec = (0..5)
            .map(|index| Bar {
                low: Num::from(index * 100),
                high: Num::from(index * 200),
                ..Default::default()
            })
            .collect::<Vec<_>>();
        let bars = Bars {
            bars: bar_vec,
            ..Default::default()
        };

        let median = bars.average_median();
        assert_eq!(median, Some(Num::from(300)))
    }

    #[test]
    fn close_median() {
        let bar_vec = (1..=5)
            .map(|index| Bar {
                close: Num::from(index * 100),
                ..Default::default()
            })
            .collect::<Vec<_>>();
        let bars = Bars {
            bars: bar_vec,
            ..Default::default()
        };

        let median = bars.close_median();
        assert_eq!(median, Some(Num::from(300)))
    }

    #[test]
    fn close_median_empty() {
        let bars = Bars {
            ..Default::default()
        };

        let median = bars.close_median();
        assert_eq!(median, None)
    }

    #[test]
    fn period_bar_empty() {
        let bars = Bars {
            ..Default::default()
        };
        let period_bar = bars.period_bar();
        assert_eq!(period_bar, None)
    }

    #[test]
    fn period_bar_several_bars() {
        let bar_vec = (0..5)
            .map(|index| Bar {
                timestamp: date(index),
                close: Num::from(index * 200),
                open: Num::from(index * 100),
                ..Default::default()
            })
            .collect::<Vec<_>>();
        let bars = Bars {
            bars: bar_vec,
            ..Default::default()
        };

        let period_bar = bars.period_bar();
        let expected = Bar {
            timestamp: date(0),
            open: Num::from(0),
            close: Num::from(800),
            ..Default::default()
        };
        assert_eq!(period_bar, Some(expected))
    }
    fn date(min: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2023, 12, 25, 0, min, 0)
            .earliest()
            .expect("failed to create test date")
    }
}
