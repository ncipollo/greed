#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum TimeFrame {
    OneMinute,
    OneHour,
    #[default]
    OneDay,
}

impl TimeFrame {
    /// Stable name used to key stored bars (matches Alpaca's timeframe names).
    pub fn as_str(&self) -> &'static str {
        match self {
            TimeFrame::OneMinute => "1Min",
            TimeFrame::OneHour => "1Hour",
            TimeFrame::OneDay => "1Day",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn as_str_names_each_timeframe() {
        assert_eq!(TimeFrame::OneMinute.as_str(), "1Min");
        assert_eq!(TimeFrame::OneHour.as_str(), "1Hour");
        assert_eq!(TimeFrame::OneDay.as_str(), "1Day");
    }

    #[test]
    fn default() {
        let time_frame: TimeFrame = Default::default();
        assert_eq!(time_frame, TimeFrame::OneDay)
    }
}
