use crate::asset::AssetSymbol;
use crate::platform::bar::Bar;
use crate::statistics::median;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Bars {
    pub symbol: AssetSymbol,
    pub bars: Vec<Bar>,
}

impl Bars {
    pub fn with_bars(bars: Vec<Bar>) -> Self {
        Self {
            symbol: Default::default(),
            bars,
        }
    }
    pub fn average_median(&self) -> Option<f64> {
        self.median(|b| b.average())
    }

    pub fn close_median(&self) -> Option<f64> {
        self.median(|b| b.close)
    }

    fn median<F>(&self, func: F) -> Option<f64>
    where
        F: FnMut(&Bar) -> f64,
    {
        if self.is_empty() {
            return None;
        }
        let values = self.bars.iter().map(func).collect::<Vec<_>>();
        return median(values);
    }

    pub fn positive_percent_median(&self) -> Option<f64> {
        let average_median = self.average_median().unwrap_or_default();
        let positive_percents = self
            .bars
            .iter()
            .map(|b| b.displacement_from_value_percent(average_median))
            .filter(|&p| p >= 0.0)
            .collect::<Vec<_>>();
        median(positive_percents)
    }

    pub fn negative_percent_median(&self) -> Option<f64> {
        let average_median = self.average_median().unwrap_or_default();
        let negative_percents = self
            .bars
            .iter()
            .map(|b| b.displacement_from_value_percent(average_median))
            .filter(|&p| p <= 0.0)
            .collect::<Vec<_>>();
        median(negative_percents)
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

    #[cfg(test)]
    pub fn fixture(symbol: AssetSymbol, base_average: f64) -> Self {
        Self {
            symbol,
            bars: vec![
                Bar::fixture(base_average),
                Bar::fixture(base_average + 100.0),
                Bar::fixture(base_average + 200.0),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, TimeZone, Utc};

    use super::*;

    #[test]
    fn average_median() {
        let bar_vec = (0..5)
            .map(|index| Bar {
                low: f64::from(index) * 100.0,
                high: f64::from(index) * 200.0,
                ..Default::default()
            })
            .collect::<Vec<_>>();
        let bars = Bars {
            bars: bar_vec,
            ..Default::default()
        };

        let median = bars.average_median();
        assert_eq!(median, Some(300.0))
    }

    #[test]
    fn close_median() {
        let bar_vec = (1..=5)
            .map(|index| Bar {
                close: f64::from(index) * 100.0,
                ..Default::default()
            })
            .collect::<Vec<_>>();
        let bars = Bars {
            bars: bar_vec,
            ..Default::default()
        };

        let median = bars.close_median();
        assert_eq!(median, Some(300.0))
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
    fn positive_percent_median() {
        let bar_vec = (0..5)
            .map(|index| Bar {
                close: f64::from(index) * 150.0,
                low: f64::from(index) * 100.0,
                high: f64::from(index) * 200.0,
                ..Default::default()
            })
            .collect::<Vec<_>>();
        let bars = Bars {
            bars: bar_vec,
            ..Default::default()
        };

        let median = bars.positive_percent_median();
        assert_eq!(median, Some(50.0))
    }

    #[test]
    fn negative_percent_median() {
        let bar_vec = (0..5)
            .map(|index| Bar {
                low: f64::from(index + 1) * 200.0,
                high: f64::from(index + 1) * 100.0,
                ..Default::default()
            })
            .collect::<Vec<_>>();
        let bars = Bars {
            bars: bar_vec,
            ..Default::default()
        };

        let median = bars.negative_percent_median();
        assert_eq!(median, Some(-100.0))
    }

    #[test]
    fn positive_negative_median() {
        let bar_vec = (0..5)
            .map(|index| Bar {
                close: f64::from(5 + index) * 200.0,
                low: f64::from(index) * 200.0,
                high: f64::from(index) * 100.0,
                ..Default::default()
            })
            .collect::<Vec<_>>();
        let bars = Bars {
            bars: bar_vec,
            ..Default::default()
        };

        let median = bars.negative_percent_median();
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
                close: (index as f64) * 200.0,
                open: (index as f64) * 100.0,
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
            open: 0.0,
            close: 800.0,
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
