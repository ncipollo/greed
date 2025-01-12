use clap::Args;
use greed::config::platform::PlatformType;
use greed::platform::args::PlatformArgs;

#[derive(Args, Debug)]
pub struct OrdersArgs {
    /// Indicates if we should use a simulated financial platform instead of a live account.
    #[arg(short = 's', long)]
    pub is_simulated: bool,
    #[arg(short = 'p', long, default_value="alpaca")]
    pub platform_type: PlatformType
}

impl From<&OrdersArgs> for PlatformArgs {
    fn from(value: &OrdersArgs) -> Self {
        PlatformArgs {
            is_simulated: value.is_simulated,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from() {
        let orders_args = OrdersArgs {
            is_simulated: true,
            platform_type: Default::default(),
        };
        let platform_args: PlatformArgs = PlatformArgs::from(&orders_args);
        let expected = PlatformArgs {
            is_simulated: true,
        };
        assert_eq!(platform_args, expected)
    }
}