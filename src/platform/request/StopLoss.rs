use num_decimal::Num;
use crate::platform::request::StopLoss::StopLoss::Stop;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StopLoss {
    /// The stop loss price to use.
    Stop(Num),
    /// The stop loss and stop limit price to use.
    StopLimit(Num, Num),
}

impl Default for StopLoss {
    fn default() -> Self {
        Stop(Num::from(0))
    }
}

#[cfg(test)]
mod test {
    use num_decimal::Num;
    use crate::platform::request::StopLoss::StopLoss;

    #[test]
    fn default() {
        let stop_loss: StopLoss = Default::default();
        assert_eq!(stop_loss, StopLoss::Stop(Num::from(0)))
    }
}