use clap::{CommandFactory, Parser};
use log::LevelFilter;
use simplelog::{ColorChoice, CombinedLogger, Config, ConfigBuilder, TerminalMode, TermLogger};

use greed::{analyze_stocks, fetch_quote, fetch_recent_orders, fetch_status, greed_loop};
use greed::platform::args::PlatformArgs;

use crate::cli::{Cli, Command};

mod cli;

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
        Command::Analyze(args) => {
            analyze_stocks(
                &args.symbols,
                &args.platform_type,
                PlatformArgs::from(&args),
            )
                .await
                .expect("stock analysis failed");
        }
        Command::Orders(args) => {
            fetch_recent_orders(PlatformArgs::from(&args), &args.platform_type)
                .await
                .expect("status fetch failed");
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
        Command::Run(args) => {
            setup_logging(log_config);
            let result = greed_loop(args.into()).await;
            if let Err(e) = result {
                panic!("{}", e);
            }
            ()
        }
        Command::Status(args) => {
            let platform_args = PlatformArgs::from(&args);
            fetch_status(platform_args, &args.platform_type, args.full)
                .await
                .expect("status fetch failed");
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
