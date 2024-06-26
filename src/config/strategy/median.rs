use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MedianPeriod {
    Day,
    Week,
    Month,
}

impl Default for MedianPeriod {
    fn default() -> Self {
        Self::Month
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default() {
        assert_eq!(MedianPeriod::Month, Default::default())
    }
}
