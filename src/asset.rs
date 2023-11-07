use serde::de::{Error, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

/// A symbol representing an asset (for example a stock such as VTI).
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct AssetSymbol {
    pub symbol: String,
}

impl AssetSymbol {
    pub fn new(symbol: &str) -> Self {
        Self {
            symbol: symbol.to_string(),
        }
    }
}

impl FromStr for AssetSymbol {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let symbol = s.trim_start_matches("$").to_string();
        Ok(AssetSymbol { symbol })
    }
}

impl From<String> for AssetSymbol {
    fn from(value: String) -> Self {
        AssetSymbol::from_str(&value).expect("asset symbol from_str shouldn't fail")
    }
}

struct AssetVisitor;

impl<'de> Visitor<'de> for AssetVisitor {
    type Value = AssetSymbol;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("expecting a string with or without a $ prefix")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        // This parser never returns an error so we can just unwrap.
        let asset = v.parse::<AssetSymbol>().unwrap();
        Ok(asset)
    }
}

impl<'de> Deserialize<'de> for AssetSymbol {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(AssetVisitor)
    }
}

impl Serialize for AssetSymbol {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.symbol)
    }
}

impl Display for AssetSymbol {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.symbol.to_uppercase())
    }
}

#[cfg(test)]
mod test {
    use crate::asset::AssetSymbol;
    use serde::{Deserialize, Serialize};
    use serde_json::json;

    #[test]
    fn display() {
        let asset = "vti".parse::<AssetSymbol>().expect("failed to parse asset");
        let display = asset.to_string();
        assert_eq!(display, "VTI")
    }

    #[test]
    fn from_str_no_dollar_sign() {
        let asset = "vti".parse::<AssetSymbol>().expect("failed to parse asset");
        let expected = AssetSymbol::new("vti");
        assert_eq!(asset, expected);
    }

    #[test]
    fn from_str_with_dollar_sign() {
        let asset = "$vti"
            .parse::<AssetSymbol>()
            .expect("failed to parse asset");
        let expected = AssetSymbol::new("vti");
        assert_eq!(asset, expected);
    }

    #[derive(Debug, Deserialize, PartialEq, Serialize)]
    struct TestAssetContainer {
        asset: AssetSymbol,
    }

    #[test]
    fn deserialize() {
        let json = json!({
            "asset": "$vti"
        });
        let container =
            serde_json::from_value::<TestAssetContainer>(json).expect("failed to deserialize");
        let expected = TestAssetContainer {
            asset: AssetSymbol::new("vti"),
        };
        assert_eq!(expected, container)
    }

    #[test]
    fn serialize() {
        let test_container = TestAssetContainer {
            asset: AssetSymbol {
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
