use crate::cli::history::range::HistoryRangeArgs;
use clap::Args;
use greed::asset::AssetSymbol;

#[derive(Args, Debug)]
pub struct QuoteHistoryArgs {
    /// Asset symbols to filter to; omit to include all symbols.
    #[arg(value_name = "SYMBOLS")]
    pub symbols: Vec<AssetSymbol>,
    #[command(flatten)]
    pub history_range: HistoryRangeArgs,
}

#[cfg(test)]
mod test {
    use crate::cli::Cli;
    use clap::Parser;
    use greed::asset::AssetSymbol;

    #[test]
    fn parses_with_no_symbols_defaults_to_empty_filter() {
        let cli = Cli::try_parse_from(["greed", "history", "quote-history"])
            .expect("should parse with no symbols");

        let crate::cli::Command::History(history_args) = cli.command else {
            panic!("expected History command");
        };
        let crate::cli::history::HistoryCommand::QuoteHistory(args) = history_args.command else {
            panic!("expected QuoteHistory subcommand");
        };

        assert_eq!(args.symbols, Vec::<AssetSymbol>::new());
        assert!(args.history_range.days.is_none());
    }

    #[test]
    fn parses_multiple_positional_symbols_before_range_flags() {
        let cli = Cli::try_parse_from([
            "greed",
            "history",
            "quote-history",
            "VTI",
            "AAPL",
            "--days",
            "3",
        ])
        .expect("should parse with symbols and a range flag");

        let crate::cli::Command::History(history_args) = cli.command else {
            panic!("expected History command");
        };
        let crate::cli::history::HistoryCommand::QuoteHistory(args) = history_args.command else {
            panic!("expected QuoteHistory subcommand");
        };

        assert_eq!(
            args.symbols,
            vec![AssetSymbol::new("VTI"), AssetSymbol::new("AAPL")]
        );
        assert_eq!(args.history_range.days, Some(3));
    }

    #[test]
    fn rejects_combining_days_and_last_hour() {
        let result = Cli::try_parse_from([
            "greed",
            "history",
            "quote-history",
            "--days",
            "3",
            "--last-hour",
        ]);

        assert!(result.is_err());
    }
}
