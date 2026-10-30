use anyhow::Result;
use clap::{Args, Subcommand};
use polymarket_client_sdk_v2::gamma::{self, types::request::TeamsRequest};

use crate::output::OutputFormat;
use crate::output::sports::{print_sport_types, print_sports, print_teams};

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
            print_sports(&sports, &output)?;
        }

        SportsCommand::MarketTypes => {
            let types = client.sports_market_types().await?;
            print_sport_types(&types, &output)?;
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
                .ascending(ascending)
                .league(league.into_iter().collect())
                .build();

            let teams = client.teams(&request).await?;
            print_teams(&teams, &output)?;
        }
    }

    Ok(())
}
