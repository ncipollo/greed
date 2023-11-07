#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AssetClass {
    /// Crypto currencies.
    Crypto,
    /// US equities.
    UsEquity,
    /// An unknown asset class.
    Unknown,
}

impl Default for AssetClass {
    fn default() -> Self {
        Self::Unknown
    }
}

#[cfg(test)]
mod test {
    use crate::platform::asset_class::AssetClass;

    #[test]
    fn default() {
        let default: AssetClass = Default::default();
        assert_eq!(default, AssetClass::Unknown)
    }
}
