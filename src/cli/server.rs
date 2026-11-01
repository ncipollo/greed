use crate::cli::run::CommonRunArgs;
use clap::Args;
use greed::run::GreedRunnerArgs;
use greed::server::DEFAULT_PORT;

#[derive(Args, Debug)]
pub struct ServerCommandArgs {
    #[command(flatten)]
    pub common: CommonRunArgs,
    /// Port to bind the HTTP server to on localhost.
    #[arg(long, value_name = "PORT", default_value_t = DEFAULT_PORT)]
    pub port: u16,
}

impl From<ServerCommandArgs> for GreedRunnerArgs {
    fn from(value: ServerCommandArgs) -> Self {
        value.common.into()
    }
}

#[cfg(test)]
mod test {
    use crate::cli::{Cli, Command};
    use clap::Parser;
    use greed::run::GreedRunnerArgs;
    use greed::server::DEFAULT_PORT;
    use std::path::PathBuf;

    #[test]
    fn runner_args_from() {
        let cli = Cli::try_parse_from(["greed", "server", "-s", "--record-history", "path"])
            .expect("should parse");
        let Command::Server(args) = cli.command else {
            panic!("expected Server command")
        };
        let runner_args: GreedRunnerArgs = args.into();
        let expected = GreedRunnerArgs {
            config_path: PathBuf::from("path"),
            db_path: None,
            is_simulated: true,
            record_history: true,
        };
        assert_eq!(runner_args, expected);
    }

    #[test]
    fn defaults_port_when_omitted() {
        let cli = Cli::try_parse_from(["greed", "server", "path"]).expect("should parse");
        let Command::Server(args) = cli.command else {
            panic!("expected Server command")
        };
        assert_eq!(args.port, DEFAULT_PORT);
    }

    #[test]
    fn parses_explicit_port() {
        let cli = Cli::try_parse_from(["greed", "server", "--port", "9000", "path"])
            .expect("should parse");
        let Command::Server(args) = cli.command else {
            panic!("expected Server command")
        };
        assert_eq!(args.port, 9000);
    }
}
