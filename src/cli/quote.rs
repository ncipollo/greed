use clap::Args;
use greed::asset::AssetSymbol;
use greed::config::platform::PlatformType;
use greed::platform::args::PlatformArgs;

#[derive(Args, Debug)]
pub struct QuoteArgs {
    /// Asset symbols to fetch quotes for.
    #[arg(value_name = "SYMBOLS")]
    pub symbols: Vec<AssetSymbol>,
    /// Indicates if we should use a simulated financial platform instead of a live account.
    #[arg(short = 's', long)]
    pub is_simulated: bool,
    #[arg(short = 'p', long, default_value="alpaca")]
    pub platform_type: PlatformType
}

impl From<&QuoteArgs> for PlatformArgs {
    fn from(value: &QuoteArgs) -> Self {
        PlatformArgs {
            is_simulated: value.is_simulated,
        }
    }
}

#[cfg(test)]
mod test {
    use greed::platform::args::PlatformArgs;
    use crate::cli::quote::QuoteArgs;

    #[test]
    fn from() {
        let quote_args = QuoteArgs {
            symbols: vec![],
            is_simulated: true,
            platform_type: Default::default(),
        };
        let platform_args: PlatformArgs = PlatformArgs::from(&quote_args);
        let expected = PlatformArgs {
            is_simulated: true,
        };
        assert_eq!(platform_args, expected)
    }
}