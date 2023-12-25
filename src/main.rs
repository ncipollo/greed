mod cli;

use crate::cli::{Cli, Command};
use clap::{CommandFactory, Parser};
use greed::platform::args::PlatformArgs;
use greed::{fetch_quote, greed_loop};
use log::LevelFilter;
use simplelog::{ColorChoice, CombinedLogger, Config, ConfigBuilder, TermLogger, TerminalMode};

fn main() {
    let config = create_log_config();
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async { async_main(config).await });
}

async fn async_main(log_config: Config) {
    let cli = Cli::parse();
    let command = cli.command;
    match command {
        Command::Run(args) => {
            setup_logging(log_config);
            greed_loop(args.into())
                .await
                .expect("greed loop threw error");
        }
        Command::Analyze(args) => {
            fetch_quote(
                &args.symbols,
                &args.platform_type,
                PlatformArgs::from(&args),
            )
            .await
            .expect("stock analysis");
        }
        Command::Quote(args) => {
            fetch_quote(
                &args.symbols,
                &args.platform_type,
                PlatformArgs::from(&args),
            )
            .await
            .expect("quote fetch failed");
        }
        Command::Completions { shell } => {
            shell.generate(&mut Cli::command(), &mut std::io::stdout());
        }
    }
}

fn create_log_config() -> Config {
    ConfigBuilder::new()
        .set_time_offset_to_local()
        .expect("failed to use local time for logs")
        .build()
}

fn setup_logging(config: Config) {
    CombinedLogger::init(vec![TermLogger::new(
        LevelFilter::Info,
        config,
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )])
    .expect("Failed to initialize logger")
}
