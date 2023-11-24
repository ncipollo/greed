use crate::platform::request::stop_loss::StopLoss;

impl From<apca::api::v2::order::StopLoss> for StopLoss {
    fn from(value: apca::api::v2::order::StopLoss) -> Self {
        match value {
            apca::api::v2::order::StopLoss::Stop(price) => Self::Stop(price),
            apca::api::v2::order::StopLoss::StopLimit(loss, limit) => Self::StopLimit(loss, limit),
        }
    }
}

impl From<StopLoss> for apca::api::v2::order::StopLoss {
    fn from(value: StopLoss) -> Self {
        match value {
            StopLoss::Stop(price) => Self::Stop(price),
            StopLoss::StopLimit(loss, limit) => Self::StopLimit(loss, limit),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::assert;
    use crate::platform::request::stop_loss::StopLoss;
    use num_decimal::Num;

    #[test]
    fn into() {
        assert::conversion(
            apca::api::v2::order::StopLoss::Stop(Num::from(42)),
            StopLoss::Stop(Num::from(42)),
        );
        assert::conversion(
            apca::api::v2::order::StopLoss::StopLimit(Num::from(42), Num::from(43)),
            StopLoss::StopLimit(Num::from(42), Num::from(43)),
        )
    }

    #[test]
    fn into_alpaca() {
        assert::conversion(
            StopLoss::Stop(Num::from(42)),
            apca::api::v2::order::StopLoss::Stop(Num::from(42))
        );
        assert::conversion(
            StopLoss::StopLimit(Num::from(42), Num::from(43)),
            apca::api::v2::order::StopLoss::StopLimit(Num::from(42), Num::from(43)),
        )
    }
}
