use clap::{CommandFactory, Parser};
use log::LevelFilter;
use simplelog::{ColorChoice, CombinedLogger, Config, ConfigBuilder, TermLogger, TerminalMode};

use greed::error::GreedError;
use greed::platform::args::PlatformArgs;
use greed::template;
use greed::{analyze_stocks, fetch_quote, fetch_recent_orders, fetch_status, greed_loop};

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
        Command::Init(args) => {
            generate_config_template(args)
                .await
                .expect("config template generation failed");
        }

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

async fn generate_config_template(args: cli::init::InitArgs) -> Result<(), GreedError> {
    use cli::init::InitConfigType;

    let (tmpl, filename) = match args.config_type {
        InitConfigType::Greed => (template::greed_config_template(), "greed.toml"),
        InitConfigType::Strategy => (template::strategy_config_template(), "strategy.toml"),
        InitConfigType::Agent => (template::agent_config_template(), "agent.toml"),
    };

    let output_path = match args.path {
        Some(p) if p.is_dir() => p.join(filename),
        Some(p) => p,
        None => std::env::current_dir()?.join(filename),
    };

    tokio::fs::write(&output_path, tmpl).await?;
    println!("Wrote template to {}", output_path.display());
    Ok(())
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
