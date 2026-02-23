use anyhow::Result;
use clap::{Args, Subcommand};
use polymarket_client_sdk::gamma::{self, types::request::TeamsRequest};

use crate::output::sports::{print_sport_types, print_sports_table, print_teams_table};
use crate::output::{OutputFormat, print_json};

#[derive(Args)]
pub struct SportsArgs {
    #[command(subcommand)]
    pub command: SportsCommand,
}

#[derive(Subcommand)]
pub enum SportsCommand {
    /// List supported sports
    List,

    /// List valid sports market types
    MarketTypes,

    /// List sports teams
    Teams {
        /// Max results
        #[arg(long, default_value = "25")]
        limit: i32,

        /// Pagination offset
        #[arg(long)]
        offset: Option<i32>,

        /// Sort field
        #[arg(long)]
        order: Option<String>,

        /// Sort ascending instead of descending
        #[arg(long)]
        ascending: bool,

        /// Filter by league
        #[arg(long)]
        league: Option<String>,
    },
}

pub async fn execute(client: &gamma::Client, args: SportsArgs, output: OutputFormat) -> Result<()> {
    match args.command {
        SportsCommand::List => {
            let sports = client.sports().await?;

            match output {
                OutputFormat::Table => print_sports_table(&sports),
                OutputFormat::Json => print_json(&sports)?,
            }
        }

        SportsCommand::MarketTypes => {
            let types = client.sports_market_types().await?;

            match output {
                OutputFormat::Table => print_sport_types(&types),
                OutputFormat::Json => print_json(&types)?,
            }
        }

        SportsCommand::Teams {
            limit,
            offset,
            order,
            ascending,
            league,
        } => {
            let request = TeamsRequest::builder()
                .limit(limit)
                .maybe_offset(offset)
                .maybe_order(order)
                .maybe_ascending(if ascending { Some(true) } else { None })
                .league(league.into_iter().collect::<Vec<_>>())
                .build();

            let teams = client.teams(&request).await?;

            match output {
                OutputFormat::Table => print_teams_table(&teams),
                OutputFormat::Json => print_json(&teams)?,
            }
        }
    }

    Ok(())
}
