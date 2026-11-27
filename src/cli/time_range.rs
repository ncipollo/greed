use clap::Args;
use greed::time_range::TimeRangeSpec;

/// Optional time window shared by `backfill` and `backtest`. All times are UTC.
/// With no options, the window is the last year.
#[derive(Args, Debug)]
pub struct TimeRangeArgs {
    /// Start of an explicit range, e.g. "2026-10-15" or "2026-10-15 14:00".
    #[arg(long, conflicts_with_all = ["days", "months", "years"])]
    pub from: Option<String>,
    /// End of an explicit range (requires --from); defaults to now. A bare date
    /// includes that whole day. Times in the future are clamped to now.
    #[arg(long, requires = "from")]
    pub to: Option<String>,
    /// Range covering the last N days.
    #[arg(long)]
    pub days: Option<u32>,
    /// Range covering the last N months.
    #[arg(long)]
    pub months: Option<u32>,
    /// Range covering the last N years.
    #[arg(long)]
    pub years: Option<u32>,
}

impl From<&TimeRangeArgs> for TimeRangeSpec {
    fn from(value: &TimeRangeArgs) -> Self {
        Self {
            from: value.from.clone(),
            to: value.to.clone(),
            days: value.days,
            months: value.months,
            years: value.years,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use clap::Parser;

    #[derive(Debug, Parser)]
    struct TestCli {
        #[command(flatten)]
        range: TimeRangeArgs,
    }

    fn parse(args: &[&str]) -> Result<TestCli, clap::Error> {
        TestCli::try_parse_from(std::iter::once("test").chain(args.iter().copied()))
    }

    #[test]
    fn no_options_converts_to_default_spec() {
        let cli = parse(&[]).expect("should parse");
        assert_eq!(TimeRangeSpec::from(&cli.range), TimeRangeSpec::default());
    }

    #[test]
    fn explicit_range_converts() {
        let cli = parse(&["--from", "2026-01-01", "--to", "2026-02-01 10:00"]).expect("parse");
        let expected = TimeRangeSpec {
            from: Some("2026-01-01".to_string()),
            to: Some("2026-02-01 10:00".to_string()),
            ..Default::default()
        };
        assert_eq!(TimeRangeSpec::from(&cli.range), expected);
    }

    #[test]
    fn relative_options_can_be_combined() {
        let cli = parse(&["--years", "1", "--months", "6", "--days", "2"]).expect("parse");
        let expected = TimeRangeSpec {
            years: Some(1),
            months: Some(6),
            days: Some(2),
            ..Default::default()
        };
        assert_eq!(TimeRangeSpec::from(&cli.range), expected);
    }

    #[test]
    fn to_without_from_is_rejected() {
        assert!(parse(&["--to", "2026-01-01"]).is_err());
    }

    #[test]
    fn from_with_relative_option_is_rejected() {
        assert!(parse(&["--from", "2026-01-01", "--days", "3"]).is_err());
        assert!(parse(&["--from", "2026-01-01", "--months", "3"]).is_err());
        assert!(parse(&["--from", "2026-01-01", "--years", "3"]).is_err());
    }

    #[test]
    fn negative_relative_value_is_rejected() {
        assert!(parse(&["--days", "-1"]).is_err());
    }
}
