use crate::platform::asset_class::AssetClass;

impl From<apca::api::v2::asset::Class> for AssetClass {
    fn from(value: apca::api::v2::asset::Class) -> Self {
        match value {
            apca::api::v2::asset::Class::UsEquity => Self::UsEquity,
            apca::api::v2::asset::Class::Crypto => Self::Crypto,
            _ => Self::Unknown,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::platform::asset_class::AssetClass;

    #[test]
    fn into_crypto() {
        let class: AssetClass = apca::api::v2::asset::Class::Crypto.into();
        assert_eq!(class, AssetClass::Crypto)
    }

    #[test]
    fn into_us_equity() {
        let class: AssetClass = apca::api::v2::asset::Class::UsEquity.into();
        assert_eq!(class, AssetClass::UsEquity)
    }

    #[test]
    fn into_unknown() {
        let class: AssetClass = apca::api::v2::asset::Class::Unknown.into();
        assert_eq!(class, AssetClass::Unknown)
    }
}
