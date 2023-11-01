use clap::Args;
use greed::run::GreedRunnerArgs;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct RunCommandArgs {
    /// Path to the greed configuration toml file
    #[arg(value_name = "PATH")]
    config_path: PathBuf,
    #[arg(short = 's', long)]
    is_simulated: bool
}

impl From<RunCommandArgs> for GreedRunnerArgs {
    fn from(value: RunCommandArgs) -> Self {
        Self {
            config_path: value.config_path.clone(),
            is_simulated: value.is_simulated,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::cli::run::RunCommandArgs;
    use greed::run::GreedRunnerArgs;
    use std::path::PathBuf;

    #[test]
    fn from() {
        let command_args = RunCommandArgs {
            config_path: PathBuf::from("path"),
            is_simulated: true
        };
        let runner_args: GreedRunnerArgs = command_args.into();
        let expected = GreedRunnerArgs {
            config_path: PathBuf::from("path"),
            is_simulated: true,
        };
        assert_eq!(runner_args, expected)
    }
}
