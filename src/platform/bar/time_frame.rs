#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TimeFrame {
    OneMinute,
    OneHour,
    OneDay,
}

impl Default for TimeFrame {
    fn default() -> Self {
        TimeFrame::OneDay
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default() {
        let time_frame: TimeFrame = Default::default();
        assert_eq!(time_frame, TimeFrame::OneDay)
    }
}
