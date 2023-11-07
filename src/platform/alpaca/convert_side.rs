use crate::platform::side::Side;

impl From<apca::api::v2::position::Side> for Side {
    fn from(value: apca::api::v2::position::Side) -> Self {
        match value {
            apca::api::v2::position::Side::Long => {
                Self::Long
            }
            apca::api::v2::position::Side::Short => {
                Self::Short
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::platform::side::Side;

    #[test]
    fn into_long() {
        let side: Side = apca::api::v2::position::Side::Long.into();
        assert_eq!(side, Side::Long)
    }

    #[test]
    fn into_short() {
        let side: Side = apca::api::v2::position::Side::Short.into();
        assert_eq!(side, Side::Short)
    }
}