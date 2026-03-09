#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum AssetClass {
    /// Crypto currencies.
    Crypto,
    /// US equities.
    UsEquity,
    /// An unknown asset class.
    #[default]
    Unknown,
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
