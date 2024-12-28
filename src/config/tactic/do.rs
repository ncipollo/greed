use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(untagged)]
pub enum DoConfig {
    Buy { buy_percent: f64 },
    SellAll { sell_all: bool },
}

impl Default for DoConfig {
    fn default() -> Self {
        Self::Buy { buy_percent: 0.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default() {
        assert_eq!(DoConfig::Buy { buy_percent: 0.0 }, Default::default())
    }
}
