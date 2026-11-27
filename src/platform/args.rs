use crate::run::GreedRunnerArgs;

#[derive(Debug, Default, Eq, PartialEq)]
pub struct PlatformArgs {
    pub is_simulated: bool,
    /// Replaces the platform's default request limit, for accounts on a plan
    /// with a higher limit than the platform's free-tier default.
    pub requests_per_minute: Option<u32>,
}

impl From<GreedRunnerArgs> for PlatformArgs {
    fn from(value: GreedRunnerArgs) -> Self {
        Self {
            is_simulated: value.is_simulated,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod test {
    use crate::platform::args::PlatformArgs;
    use crate::run::GreedRunnerArgs;

    #[test]
    fn from() {
        let runner_args = GreedRunnerArgs {
            config_path: Default::default(),
            db_path: None,
            is_simulated: true,
            record_history: false,
        };
        let platform_args: PlatformArgs = runner_args.into();
        let expected = PlatformArgs {
            is_simulated: true,
            ..Default::default()
        };
        assert_eq!(platform_args, expected)
    }

    #[test]
    fn default_has_no_rate_limit_override() {
        assert_eq!(PlatformArgs::default().requests_per_minute, None);
    }
}
