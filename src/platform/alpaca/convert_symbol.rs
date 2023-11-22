use apca::api::v2::asset::Symbol;
use crate::asset::AssetSymbol;

impl From<AssetSymbol> for Symbol {
    fn from(value: AssetSymbol) -> Self {
        Self::Sym(value.to_string())
    }
}

#[cfg(test)]
mod test {
    use apca::api::v2::asset::Symbol;
    use crate::assert;
    use crate::asset::AssetSymbol;

    #[test]
    fn into_alpaca() {
        let asset_symbol = AssetSymbol::new("VTI");
        assert::conversion(asset_symbol, Symbol::Sym("VTI".to_string()))
    }
}