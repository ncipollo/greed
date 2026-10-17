use clap::Args;
use greed::history_range::HistoryRange;

#[derive(Args, Debug)]
pub struct HistoryRangeArgs {
    /// Start of an explicit range, e.g. "2026-10-15" or "2026-10-15 14:00".
    #[arg(long, group = "history_range")]
    pub from: Option<String>,
    /// End of an explicit range (requires --from); defaults to now if omitted.
    #[arg(long, requires = "from")]
    pub to: Option<String>,
    /// Shorthand for the last hour.
    #[arg(long, group = "history_range")]
    pub last_hour: bool,
    /// Shorthand for the last N hours.
    #[arg(long, group = "history_range")]
    pub hours: Option<i64>,
    /// Shorthand for the last N days.
    #[arg(long, group = "history_range")]
    pub days: Option<i64>,
}

impl From<&HistoryRangeArgs> for HistoryRange {
    fn from(value: &HistoryRangeArgs) -> Self {
        Self {
            from: value.from.clone(),
            to: value.to.clone(),
            last_hour: value.last_hour,
            hours: value.hours,
            days: value.days,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from() {
        let args = HistoryRangeArgs {
            from: Some("2026-10-01".to_string()),
            to: None,
            last_hour: false,
            hours: Some(3),
            days: None,
        };
        let range: HistoryRange = HistoryRange::from(&args);
        let expected = HistoryRange {
            from: Some("2026-10-01".to_string()),
            hours: Some(3),
            ..Default::default()
        };
        assert_eq!(range, expected);
    }
}
