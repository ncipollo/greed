use serde::de::{Error, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::Formatter;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub struct Asset {
    pub symbol: String,
}

impl Asset {
    fn new(symbol: &str) -> Self {
        Self {
            symbol: symbol.to_string(),
        }
    }
}

impl FromStr for Asset {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let symbol = s.trim_start_matches("$").to_string();
        Ok(Asset { symbol })
    }
}

struct AssetVisitor;

impl<'de> Visitor<'de> for AssetVisitor {
    type Value = Asset;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("expecting a string with or without a $ prefix")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        // This parser never returns an error so we can just unwrap.
        let asset = v.parse::<Asset>().unwrap();
        Ok(asset)
    }
}

impl<'de> Deserialize<'de> for Asset {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(AssetVisitor)
    }
}

impl Serialize for Asset {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.symbol)
    }
}

#[cfg(test)]
mod test {
    use crate::asset::Asset;
    use serde::{Deserialize, Serialize};
    use serde_json::json;

    #[test]
    fn from_str_no_dollar_sign() {
        let asset = "vti".parse::<Asset>().expect("failed to parse asset");
        let expected = Asset::new("vti");
        assert_eq!(asset, expected);
    }

    #[test]
    fn from_str_with_dollar_sign() {
        let asset = "$vti".parse::<Asset>().expect("failed to parse asset");
        let expected = Asset::new("vti");
        assert_eq!(asset, expected);
    }

    #[derive(Debug, Deserialize, PartialEq, Serialize)]
    struct TestAssetContainer {
        asset: Asset,
    }

    #[test]
    fn deserialize() {
        let json = json!({
            "asset": "$vti"
        });
        let container =
            serde_json::from_value::<TestAssetContainer>(json).expect("failed to deserialize");
        let expected = TestAssetContainer {
            asset: Asset::new("vti"),
        };
        assert_eq!(expected, container)
    }

    #[test]
    fn serialize() {
        let test_container = TestAssetContainer {
            asset: Asset {
                symbol: "vti".to_string(),
            },
        };
        let expected_json = json!({
            "asset": "vti"
        });
        let expected = serde_json::to_string_pretty(&expected_json).unwrap();
        let json = serde_json::to_string_pretty(&test_container).expect("failed to serialize");

        assert_eq!(json, expected)
    }
}
