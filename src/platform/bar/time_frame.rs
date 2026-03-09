#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum TimeFrame {
    OneMinute,
    OneHour,
    #[default]
    OneDay,
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
