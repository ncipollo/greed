use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, ValueEnum)]
#[derive(Default)]
#[serde(rename_all = "snake_case")]
pub enum PlatformType {
    #[default]
    Alpaca,
}

#[cfg(test)]
mod test {
    use crate::config::platform::PlatformType;

    #[test]
    fn default() {
        let default = PlatformType::default();
        assert_eq!(PlatformType::Alpaca, default)
    }
}
