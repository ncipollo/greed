use crate::config::tactic::median::MedianPeriod;
use crate::config::quote_fetcher_config::QuoteFetcherConfig;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(untagged)]
pub enum WhenConfig {
    AllOf {
        all_off: Vec<WhenConfig>,
    },
    Always {
        always: bool,
    },
    BelowMedian {
        below_median_percent: f64,
        #[serde(default)]
        median_period: MedianPeriod,
    },
    GainAbove {
        gain_above_percent: f64,
    },
    Never {
        never: bool,
    },
}

impl Default for WhenConfig {
    fn default() -> Self {
        Self::Always { always: true }
    }
}

impl QuoteFetcherConfig for WhenConfig {
    fn should_fetch_quotes(&self) -> bool {
        match self {
            WhenConfig::AllOf { all_off } => all_off.iter().any(|config| config.should_fetch_quotes()),
            WhenConfig::Always { .. } => false,
            WhenConfig::BelowMedian { .. } => true,
            WhenConfig::GainAbove { .. } => false,
            WhenConfig::Never { .. } => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default() {
        assert_eq!(WhenConfig::Always { always: true }, Default::default())
    }

    #[test]
    fn should_fetch_quotes_all_of() {
        let all_of = WhenConfig::AllOf {
            all_off: vec![
                WhenConfig::Always { always: true },
                WhenConfig::BelowMedian { below_median_percent: 10.0, median_period: MedianPeriod::default() },
            ],
        };
        assert!(all_of.should_fetch_quotes());
    }

    #[test]
    fn should_fetch_quotes_all_of_without_quotes() {
        let all_of = WhenConfig::AllOf {
            all_off: vec![
                WhenConfig::Always { always: true },
                WhenConfig::GainAbove { gain_above_percent: 10.0 },
                WhenConfig::Never { never: true },
            ],
        };
        assert!(!all_of.should_fetch_quotes());
    }

    #[test]
    fn should_fetch_quotes_always() {
        let always = WhenConfig::Always { always: true };
        assert!(!always.should_fetch_quotes());
    }

    #[test]
    fn should_fetch_quotes_below_median() {
        let below_median = WhenConfig::BelowMedian {
            below_median_percent: 10.0,
            median_period: MedianPeriod::default(),
        };
        assert!(below_median.should_fetch_quotes());
    }

    #[test]
    fn should_fetch_quotes_gain_above() {
        let gain_above = WhenConfig::GainAbove { gain_above_percent: 10.0 };
        assert!(!gain_above.should_fetch_quotes());
    }

    #[test]
    fn should_fetch_quotes_never() {
        let never = WhenConfig::Never { never: true };
        assert!(!never.should_fetch_quotes());
    }
}
