use crate::platform::request::stop_loss::StopLoss;

impl From<apca::api::v2::order::StopLoss> for StopLoss {
    fn from(value: apca::api::v2::order::StopLoss) -> Self {
        match value {
            apca::api::v2::order::StopLoss::Stop(price) => Self::Stop(price),
            apca::api::v2::order::StopLoss::StopLimit(loss, limit) => Self::StopLimit(loss, limit),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::assert;
    use num_decimal::Num;
    use crate::platform::request::stop_loss::StopLoss;

    #[test]
    fn into() {
        assert::conversion(
            apca::api::v2::order::StopLoss::Stop(Num::from(42)),
            StopLoss::Stop(Num::from(42))
        );
        assert::conversion(
            apca::api::v2::order::StopLoss::StopLimit(Num::from(42), Num::from(43)),
            StopLoss::StopLimit(Num::from(42), Num::from(43))
        )
    }
}
