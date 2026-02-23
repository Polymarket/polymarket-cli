#![allow(clippy::items_after_statements)]

mod auth;
mod commands;
mod config;
mod output;

use std::process::ExitCode;

use clap::{Parser, Subcommand};
use output::OutputFormat;

#[derive(Parser)]
#[command(name = "polymarket", about = "Polymarket CLI", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output format: table or json
    #[arg(short, long, global = true, default_value = "table")]
    output: OutputFormat,

    /// Private key (overrides env var and config file)
    #[arg(long, global = true)]
    private_key: Option<String>,

    /// Signature type: eoa, proxy, or gnosis-safe
    #[arg(long, global = true)]
    signature_type: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
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
    /// Interact with the CLOB (order book, trading, balances)
    Clob(commands::clob::ClobArgs),
    /// Query on-chain data (positions, trades, leaderboards)
    Data(commands::data::DataArgs),
    /// Bridge assets from other chains to Polymarket
    Bridge(commands::bridge::BridgeArgs),
    /// Manage wallet and authentication
    Wallet(commands::wallet::WalletArgs),
    /// Check API health status
    Status,
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    let output = cli.output.clone();

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

#[allow(clippy::too_many_lines)]
async fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
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
        Commands::Clob(args) => {
            commands::clob::execute(
                args,
                cli.output,
                cli.private_key.as_deref(),
                cli.signature_type.as_deref(),
            )
            .await
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
