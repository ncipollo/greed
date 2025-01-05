use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(untagged)]
pub enum StrategyConfig {
    LocalFile {
        path: String,
        #[serde(flatten)]
        properties: StrategyProperties,
    },
}

impl StrategyConfig {
    pub fn properties(&self) -> StrategyProperties {
        match self {
            Self::LocalFile { properties, .. } => properties.clone(),
        }
    }
}

impl Default for StrategyConfig {
    fn default() -> Self {
        Self::LocalFile {
            path: "".to_string(),
            properties: Default::default(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct StrategyProperties {
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_portfolio_percent")]
    pub portfolio_percent: f64,
}

fn default_portfolio_percent() -> f64 {
    100.0
}

impl Default for StrategyProperties {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            portfolio_percent: default_portfolio_percent(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default() {
        let default = StrategyConfig::default();
        let expected_props = StrategyProperties {
            name: "".to_string(),
            portfolio_percent: 100.0,
        };
        let expected = StrategyConfig::LocalFile {
            path: "".to_string(),
            properties: expected_props,
        };
        assert_eq!(expected, default)
    }

    #[test]
    fn default_portfolio_percent() {
        assert_eq!(100.0, super::default_portfolio_percent())
    }

    #[test]
    fn properties_local_file() {
        let config = StrategyConfig::LocalFile {
            path: "test".to_string(),
            properties: test_properties(),
        };
        assert_eq!(test_properties(), config.properties())
    }

    fn test_properties() -> StrategyProperties {
        StrategyProperties {
            name: "test".to_string(),
            portfolio_percent: 50.0,
        }
    }
}
