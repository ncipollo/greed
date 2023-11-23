use crate::platform::order::time_in_force::TimeInForce;

impl From<apca::api::v2::order::TimeInForce> for TimeInForce {
    fn from(value: apca::api::v2::order::TimeInForce) -> Self {
        match value {
            apca::api::v2::order::TimeInForce::Day => TimeInForce::Day,
            apca::api::v2::order::TimeInForce::FillOrKill => TimeInForce::FillOrKill,
            apca::api::v2::order::TimeInForce::ImmediateOrCancel => TimeInForce::ImmediateOrCancel,
            apca::api::v2::order::TimeInForce::UntilCanceled => TimeInForce::UntilCanceled,
            apca::api::v2::order::TimeInForce::UntilMarketOpen => TimeInForce::UntilMarketOpen,
            apca::api::v2::order::TimeInForce::UntilMarketClose => TimeInForce::UntilMarketClose,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::assert;
    use crate::platform::order::time_in_force::TimeInForce;

    #[test]
    fn into() {
        assert::conversion(apca::api::v2::order::TimeInForce::Day, TimeInForce::Day);
        assert::conversion(
            apca::api::v2::order::TimeInForce::FillOrKill,
            TimeInForce::FillOrKill,
        );
        assert::conversion(
            apca::api::v2::order::TimeInForce::ImmediateOrCancel,
            TimeInForce::ImmediateOrCancel,
        );
        assert::conversion(
            apca::api::v2::order::TimeInForce::UntilCanceled,
            TimeInForce::UntilCanceled,
        );
        assert::conversion(
            apca::api::v2::order::TimeInForce::UntilMarketOpen,
            TimeInForce::UntilMarketOpen,
        );
        assert::conversion(
            apca::api::v2::order::TimeInForce::UntilMarketClose,
            TimeInForce::UntilMarketClose,
        );
    }
}
