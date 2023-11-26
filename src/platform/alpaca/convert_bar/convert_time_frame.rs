use crate::platform::bar::time_frame::TimeFrame;

impl From<TimeFrame> for apca::data::v2::bars::TimeFrame {
    fn from(value: TimeFrame) -> Self {
        match value {
            TimeFrame::OneMinute => Self::OneMinute,
            TimeFrame::OneHour => Self::OneHour,
            TimeFrame::OneDay => Self::OneDay
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::assert;
    use super::*;

    #[test]
    fn into_alpaca() {
        assert::conversion(TimeFrame::OneMinute, apca::data::v2::bars::TimeFrame::OneMinute);
        assert::conversion(TimeFrame::OneHour, apca::data::v2::bars::TimeFrame::OneHour);
        assert::conversion(TimeFrame::OneDay, apca::data::v2::bars::TimeFrame::OneDay);
    }
}