use clap::Args;
use greed::config::platform::PlatformType;
use greed::platform::args::PlatformArgs;

#[derive(Args, Debug)]
pub struct StatusArgs {
    /// Indicates if we should use a simulated financial platform instead of a live account.
    #[arg(short = 's', long)]
    pub is_simulated: bool,
    #[arg(short = 'p', long, default_value="alpaca")]
    pub platform_type: PlatformType
}

impl From<&StatusArgs> for PlatformArgs {
    fn from(value: &StatusArgs) -> Self {
        PlatformArgs {
            is_simulated: value.is_simulated,
        }
    }
}

#[cfg(test)]
mod test {
    use greed::platform::args::PlatformArgs;
    use crate::cli::status::StatusArgs;

    #[test]
    fn from() {
        let status_args = StatusArgs {
            is_simulated: true,
            platform_type: Default::default(),
        };
        let platform_args: PlatformArgs = PlatformArgs::from(&status_args);
        let expected = PlatformArgs {
            is_simulated: true,
        };
        assert_eq!(platform_args, expected)
    }
}