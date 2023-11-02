use crate::run::GreedRunnerArgs;

#[derive(Debug, Eq, PartialEq)]
pub struct PlatformArgs {
    pub is_simulated: bool
}

impl From<GreedRunnerArgs> for PlatformArgs {
    fn from(value: GreedRunnerArgs) -> Self {
        Self {
            is_simulated: value.is_simulated
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
            is_simulated: true,
        };
        let platform_args: PlatformArgs = runner_args.into();
        let expected = PlatformArgs {
            is_simulated: true
        };
        assert_eq!(platform_args, expected)
    }
}