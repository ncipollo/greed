use crate::asset::AssetSymbol;
use apca::api::v2::asset::Symbol;

impl From<AssetSymbol> for Symbol {
    fn from(value: AssetSymbol) -> Self {
        Self::Sym(value.to_string())
    }
}

#[cfg(test)]
mod test {
    use crate::assert;
    use crate::asset::AssetSymbol;
    use apca::api::v2::asset::Symbol;

    #[test]
    fn into_alpaca() {
        let asset_symbol = AssetSymbol::new("VTI");
        assert::conversion(asset_symbol, Symbol::Sym("VTI".to_string()))
    }
}
