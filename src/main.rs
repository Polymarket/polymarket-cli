mod auth;
mod commands;
mod config;
mod output;
mod shell;

use std::process::ExitCode;

use clap::{Parser, Subcommand};
use output::OutputFormat;

#[derive(Parser)]
#[command(name = "polymarket", about = "Polymarket CLI", version)]
pub(crate) struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output format: table or json
    #[arg(short, long, global = true, default_value = "table")]
    pub(crate) output: OutputFormat,

    /// Auto-refresh every N seconds (table output only)
    #[arg(short, long, global = true)]
    pub(crate) watch: Option<u64>,

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
    let cli = Cli::parse();
    let output = cli.output;

    if let Some(interval) = cli.watch {
        if matches!(output, OutputFormat::Json) {
            eprintln!("Error: --watch is not supported with JSON output");
            return ExitCode::FAILURE;
        }
        if interval == 0 {
            eprintln!("Error: --watch interval must be at least 1 second");
            return ExitCode::FAILURE;
        }
        return watch_loop(interval).await;
    }

    if let Err(e) = run(cli).await {
        match output {
            OutputFormat::Json => {
                println!("{}", serde_json::json!({"error": e.to_string()}));
            }
            OutputFormat::Table => {
                eprintln!("Error: {e}");
            }
        }
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

async fn watch_loop(interval_secs: u64) -> ExitCode {
    let interval = std::time::Duration::from_secs(interval_secs);
    loop {
        print!("\x1b[2J\x1b[H");
        let now = chrono::Local::now().format("%H:%M:%S");
        println!("Every {interval_secs}s \u{2014} {now}  (Ctrl+C to stop)\n");

        let cli = Cli::parse();
        if let Err(e) = run(cli).await {
            eprintln!("Error: {e}");
        }

        tokio::select! {
            _ = tokio::time::sleep(interval) => {}
            _ = tokio::signal::ctrl_c() => {
                println!();
                return ExitCode::SUCCESS;
            }
        }
    }
}

#[allow(clippy::too_many_lines)]
pub(crate) async fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Commands::Setup => commands::setup::execute(),
        Commands::Shell => {
            Box::pin(shell::run_shell()).await;
            Ok(())
        }
        Commands::Markets(args) => {
            commands::markets::execute(
                &polymarket_client_sdk::gamma::Client::default(),
                args,
                cli.output,
            )
            .await
        }
        Commands::Events(args) => {
            commands::events::execute(
                &polymarket_client_sdk::gamma::Client::default(),
                args,
                cli.output,
            )
            .await
        }
        Commands::Tags(args) => {
            commands::tags::execute(
                &polymarket_client_sdk::gamma::Client::default(),
                args,
                cli.output,
            )
            .await
        }
        Commands::Series(args) => {
            commands::series::execute(
                &polymarket_client_sdk::gamma::Client::default(),
                args,
                cli.output,
            )
            .await
        }
        Commands::Comments(args) => {
            commands::comments::execute(
                &polymarket_client_sdk::gamma::Client::default(),
                args,
                cli.output,
            )
            .await
        }
        Commands::Profiles(args) => {
            commands::profiles::execute(
                &polymarket_client_sdk::gamma::Client::default(),
                args,
                cli.output,
            )
            .await
        }
        Commands::Sports(args) => {
            commands::sports::execute(
                &polymarket_client_sdk::gamma::Client::default(),
                args,
                cli.output,
            )
            .await
        }
        Commands::Approve(args) => {
            commands::approve::execute(args, cli.output, cli.private_key.as_deref()).await
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
            commands::ctf::execute(args, cli.output, cli.private_key.as_deref()).await
        }
        Commands::Data(args) => {
            commands::data::execute(
                &polymarket_client_sdk::data::Client::default(),
                args,
                cli.output,
            )
            .await
        }
        Commands::Bridge(args) => {
            commands::bridge::execute(
                &polymarket_client_sdk::bridge::Client::default(),
                args,
                cli.output,
            )
            .await
        }
        Commands::Wallet(args) => {
            commands::wallet::execute(args, &cli.output, cli.private_key.as_deref())
        }
        Commands::Upgrade => commands::upgrade::execute(),
        Commands::Status => {
            let status = polymarket_client_sdk::gamma::Client::default()
                .status()
                .await?;
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
