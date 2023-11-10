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
    use crate::platform::order::time_in_force::TimeInForce;

    #[test]
    fn into() {
        assert_conversion(apca::api::v2::order::TimeInForce::Day, TimeInForce::Day);
        assert_conversion(
            apca::api::v2::order::TimeInForce::FillOrKill,
            TimeInForce::FillOrKill,
        );
        assert_conversion(
            apca::api::v2::order::TimeInForce::ImmediateOrCancel,
            TimeInForce::ImmediateOrCancel,
        );
        assert_conversion(
            apca::api::v2::order::TimeInForce::UntilCanceled,
            TimeInForce::UntilCanceled,
        );
        assert_conversion(
            apca::api::v2::order::TimeInForce::UntilMarketOpen,
            TimeInForce::UntilMarketOpen,
        );
        assert_conversion(
            apca::api::v2::order::TimeInForce::UntilMarketClose,
            TimeInForce::UntilMarketClose,
        );
    }

    fn assert_conversion(alpaca_time: apca::api::v2::order::TimeInForce, expected: TimeInForce) {
        let time: TimeInForce = alpaca_time.into();
        assert_eq!(time, expected)
    }
}
