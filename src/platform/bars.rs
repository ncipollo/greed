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

    pub fn is_empty(&self) -> bool {
        self.bars.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
