use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(untagged)]
pub enum WhenConfig {
    AllOf { all_off: Vec<WhenConfig> },
    Always { always: bool },
    BelowOneDay { below_1_day_percent: f64 },
    GainAbove { gain_above_percent: f64 },
    Never { never: bool },
}

impl Default for WhenConfig {
    fn default() -> Self {
        Self::Always { always: true }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default() {
        assert_eq!(WhenConfig::Always { always: true }, Default::default())
    }
}
