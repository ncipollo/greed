use crate::cli::run::CommonRunArgs;
use clap::Args;
use greed::run::GreedRunnerArgs;
use greed::server::{DEFAULT_HOST, DEFAULT_PORT};
use std::net::IpAddr;

#[derive(Args, Debug)]
pub struct ServerCommandArgs {
    #[command(flatten)]
    pub common: CommonRunArgs,
    /// IP address to bind the HTTP server to. Defaults to loopback; use 0.0.0.0
    /// or a specific interface IP to expose it on the network (the API is
    /// unauthenticated).
    #[arg(long, value_name = "ADDR", default_value_t = DEFAULT_HOST)]
    pub host: IpAddr,
    /// Port to bind the HTTP server to.
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
    use greed::server::{DEFAULT_HOST, DEFAULT_PORT};
    use std::net::IpAddr;
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

    #[test]
    fn defaults_host_when_omitted() {
        let cli = Cli::try_parse_from(["greed", "server", "path"]).expect("should parse");
        let Command::Server(args) = cli.command else {
            panic!("expected Server command")
        };
        assert_eq!(args.host, DEFAULT_HOST);
    }

    #[test]
    fn parses_explicit_hosts() {
        for host in ["0.0.0.0", "192.168.1.20", "::"] {
            let cli = Cli::try_parse_from(["greed", "server", "--host", host, "path"])
                .expect("should parse");
            let Command::Server(args) = cli.command else {
                panic!("expected Server command")
            };
            assert_eq!(args.host, host.parse::<IpAddr>().unwrap());
        }
    }

    #[test]
    fn rejects_invalid_host() {
        assert!(Cli::try_parse_from(["greed", "server", "--host", "not-an-ip", "path"]).is_err());
    }
}
