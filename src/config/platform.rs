use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, PartialEq, Serialize)]
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
