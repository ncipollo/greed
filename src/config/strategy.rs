use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(untagged)]
pub enum StrategyConfig {
    LocalFile {
        path: String,
        #[serde(flatten)]
        options: StrategyOptions,
    },
}

impl Default for StrategyConfig {
    fn default() -> Self {
        Self::LocalFile {
            path: "".to_string(),
            options: Default::default(),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct StrategyOptions {
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_portfolio_percent")]
    pub portfolio_percent: f64,
}

fn default_portfolio_percent() -> f64 {
    100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default() {
        let default = StrategyConfig::default();
        let expected = StrategyConfig::LocalFile {
            path: "".to_string(),
            options: Default::default(),
        };
        assert_eq!(expected, default)
    }

    #[test]
    fn default_portfolio_percent() {
        assert_eq!(100.0, super::default_portfolio_percent())
    }
}
