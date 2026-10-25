mod auth;
mod commands;
mod config;
mod output;
mod shell;

use std::{ffi::OsStr, process::ExitCode};

use clap::{Parser, Subcommand, error::ErrorKind};
use output::OutputFormat;

#[derive(Parser)]
#[command(name = "polymarket", about = "Polymarket CLI", version)]
pub(crate) struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output format: table or json
    #[arg(short, long, global = true, default_value = "table")]
    pub(crate) output: OutputFormat,

    /// Private key (overrides env var and config file)
    #[arg(long, global = true)]
    private_key: Option<String>,

    /// Signature type: eoa, proxy, or gnosis-safe
    #[arg(long, global = true)]
    signature_type: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Guided first-time setup (wallet, proxy, approvals)
    Setup,
    /// Launch interactive shell
    Shell,
    /// Interact with markets
    Markets(commands::markets::MarketsArgs),
    /// Interact with events
    Events(commands::events::EventsArgs),
    /// Interact with tags
    Tags(commands::tags::TagsArgs),
    /// Interact with series
    Series(commands::series::SeriesArgs),
    /// Interact with comments
    Comments(commands::comments::CommentsArgs),
    /// Look up public profiles
    Profiles(commands::profiles::ProfilesArgs),
    /// Sports metadata and teams
    Sports(commands::sports::SportsArgs),
    /// Check and set contract approvals for trading
    Approve(commands::approve::ApproveArgs),
    /// Interact with the CLOB (order book, trading, balances)
    Clob(commands::clob::ClobArgs),
    /// CTF operations: split, merge, redeem positions
    Ctf(commands::ctf::CtfArgs),
    /// Query on-chain data (positions, trades, leaderboards)
    Data(commands::data::DataArgs),
    /// Bridge assets from other chains to Polymarket
    Bridge(commands::bridge::BridgeArgs),
    /// Manage wallet and authentication
    Wallet(commands::wallet::WalletArgs),
    /// Check API health status
    Status,
    /// Update to the latest version
    Upgrade,
}

#[tokio::main]
async fn main() -> ExitCode {
    let args: Vec<_> = std::env::args_os().collect();
    let requested_output = requested_output_format(&args[1..]);
    let cli = match Cli::try_parse_from(args) {
        Ok(cli) => cli,
        Err(error) => {
            if matches!(
                error.kind(),
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion
            ) {
                error.exit();
            }

            if matches!(requested_output, OutputFormat::Json) {
                println!(
                    "{}",
                    serde_json::json!({
                        "error": error.to_string(),
                    })
                );
                return ExitCode::from(u8::try_from(error.exit_code()).unwrap_or(1));
            }

            error.exit();
        }
    };
    let output = cli.output;

    if let Err(e) = run(cli).await {
        output::print_error(&e, output);
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

fn requested_output_format(args: &[impl AsRef<OsStr>]) -> OutputFormat {
    let mut requested = OutputFormat::Table;
    let mut args = args.iter();

    while let Some(arg) = args.next() {
        let arg = arg.as_ref();
        if arg == "--" {
            break;
        }

        if arg == "--output" || arg == "-o" {
            if args.next().is_some_and(|value| value.as_ref() == "json") {
                requested = OutputFormat::Json;
            }
            continue;
        }

        let Some(arg) = arg.to_str() else {
            continue;
        };
        if arg
            .strip_prefix("--output=")
            .is_some_and(|value| value == "json")
            || arg
                .strip_prefix("-o")
                .is_some_and(|value| !value.is_empty() && value == "json")
        {
            requested = OutputFormat::Json;
        }
    }

    requested
}

#[allow(clippy::too_many_lines)]
pub(crate) async fn run(cli: Cli) -> anyhow::Result<()> {
    // Lazy-init so we only pay for the client we actually use.
    let gamma = std::cell::LazyCell::new(polymarket_client_sdk_v2::gamma::Client::default);
    let data = std::cell::LazyCell::new(polymarket_client_sdk_v2::data::Client::default);
    let bridge = std::cell::LazyCell::new(polymarket_client_sdk_v2::bridge::Client::default);

    match cli.command {
        Commands::Setup => commands::setup::execute(),
        Commands::Shell => Box::pin(shell::run_shell()).await,
        Commands::Markets(args) => commands::markets::execute(&gamma, args, cli.output).await,
        Commands::Events(args) => commands::events::execute(&gamma, args, cli.output).await,
        Commands::Tags(args) => commands::tags::execute(&gamma, args, cli.output).await,
        Commands::Series(args) => commands::series::execute(&gamma, args, cli.output).await,
        Commands::Comments(args) => commands::comments::execute(&gamma, args, cli.output).await,
        Commands::Profiles(args) => commands::profiles::execute(&gamma, args, cli.output).await,
        Commands::Sports(args) => commands::sports::execute(&gamma, args, cli.output).await,
        Commands::Approve(args) => {
            commands::approve::execute(
                args,
                cli.output,
                cli.private_key.as_deref(),
                cli.signature_type.as_deref(),
            )
            .await
        }
        Commands::Clob(args) => {
            commands::clob::execute(
                args,
                cli.output,
                cli.private_key.as_deref(),
                cli.signature_type.as_deref(),
            )
            .await
        }
        Commands::Ctf(args) => {
            commands::ctf::execute(
                args,
                cli.output,
                cli.private_key.as_deref(),
                cli.signature_type.as_deref(),
            )
            .await
        }
        Commands::Data(args) => commands::data::execute(&data, args, cli.output).await,
        Commands::Bridge(args) => commands::bridge::execute(&bridge, args, cli.output).await,
        Commands::Wallet(args) => {
            commands::wallet::execute(args, cli.output, cli.private_key.as_deref())
        }
        Commands::Upgrade => commands::upgrade::execute(),
        Commands::Status => {
            let status = gamma.status().await?;
            match cli.output {
                OutputFormat::Json => {
                    println!("{}", serde_json::json!({"status": status}));
                }
                OutputFormat::Table => {
                    println!("API Status: {status}");
                }
            }
            Ok(())
        }
    }
}
