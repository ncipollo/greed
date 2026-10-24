use crate::cli::history::output_format::OutputFormatArgs;
use crate::cli::history::range::HistoryRangeArgs;
use clap::Args;
use greed::interval::RollupInterval;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct StatusHistoryArgs {
    #[command(flatten)]
    pub history_range: HistoryRangeArgs,
    /// Roll results up into averages per interval (e.g. "30s", "5m", "1h", "1d"); omit for raw rows.
    #[arg(long)]
    pub interval: Option<RollupInterval>,
    #[command(flatten)]
    pub output_format: OutputFormatArgs,
    /// Override the path to the DuckDB database file. Defaults to "greed.duckdb" in the current directory.
    #[arg(long, value_name = "PATH")]
    pub db_path: Option<PathBuf>,
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

    #[test]
    fn parses_explicit_db_path() {
        let cli = Cli::try_parse_from(["greed", "history", "status", "--db-path", "custom.duckdb"])
            .expect("should parse with an explicit db path");

        let crate::cli::Command::History(history_args) = cli.command else {
            panic!("expected History command");
        };
        let crate::cli::history::HistoryCommand::Status(args) = history_args.command else {
            panic!("expected Status subcommand");
        };

        assert_eq!(
            args.db_path,
            Some(std::path::PathBuf::from("custom.duckdb"))
        );
    }

    #[test]
    fn parses_explicit_interval() {
        let cli = Cli::try_parse_from(["greed", "history", "status", "--interval", "1h"])
            .expect("should parse with an explicit interval");

        let crate::cli::Command::History(history_args) = cli.command else {
            panic!("expected History command");
        };
        let crate::cli::history::HistoryCommand::Status(args) = history_args.command else {
            panic!("expected Status subcommand");
        };

        assert!(args.interval.is_some());
    }

    #[test]
    fn defaults_interval_to_none_when_omitted() {
        let cli = Cli::try_parse_from(["greed", "history", "status"])
            .expect("should parse with no interval flag");

        let crate::cli::Command::History(history_args) = cli.command else {
            panic!("expected History command");
        };
        let crate::cli::history::HistoryCommand::Status(args) = history_args.command else {
            panic!("expected Status subcommand");
        };

        assert!(args.interval.is_none());
    }

    #[test]
    fn rejects_invalid_interval_value() {
        let result = Cli::try_parse_from(["greed", "history", "status", "--interval", "bogus"]);

        assert!(result.is_err());
    }

    #[test]
    fn defaults_db_path_to_none_when_omitted() {
        let cli = Cli::try_parse_from(["greed", "history", "status"])
            .expect("should parse with no db path flag");

        let crate::cli::Command::History(history_args) = cli.command else {
            panic!("expected History command");
        };
        let crate::cli::history::HistoryCommand::Status(args) = history_args.command else {
            panic!("expected Status subcommand");
        };

        assert_eq!(args.db_path, None);
    }
}
