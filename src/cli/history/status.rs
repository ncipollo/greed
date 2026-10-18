use crate::cli::history::output_format::OutputFormatArgs;
use crate::cli::history::range::HistoryRangeArgs;
use clap::Args;

#[derive(Args, Debug)]
pub struct StatusHistoryArgs {
    #[command(flatten)]
    pub history_range: HistoryRangeArgs,
    #[command(flatten)]
    pub output_format: OutputFormatArgs,
}

#[cfg(test)]
mod test {
    use crate::cli::Cli;
    use clap::Parser;
    use greed::output_format::OutputFormat;

    #[test]
    fn defaults_to_text_when_format_omitted() {
        let cli = Cli::try_parse_from(["greed", "history", "status"])
            .expect("should parse with no format flag");

        let crate::cli::Command::History(history_args) = cli.command else {
            panic!("expected History command");
        };
        let crate::cli::history::HistoryCommand::Status(args) = history_args.command else {
            panic!("expected Status subcommand");
        };

        assert_eq!(args.output_format.format, OutputFormat::Text);
    }

    #[test]
    fn parses_explicit_json_format() {
        let cli = Cli::try_parse_from(["greed", "history", "status", "--format", "json"])
            .expect("should parse with an explicit json format");

        let crate::cli::Command::History(history_args) = cli.command else {
            panic!("expected History command");
        };
        let crate::cli::history::HistoryCommand::Status(args) = history_args.command else {
            panic!("expected Status subcommand");
        };

        assert_eq!(args.output_format.format, OutputFormat::Json);
    }

    #[test]
    fn rejects_invalid_format_value() {
        let result = Cli::try_parse_from(["greed", "history", "status", "--format", "bogus"]);

        assert!(result.is_err());
    }
}
