use anyhow::Result;
use clap::{Args, Subcommand};
use polymarket_client_sdk::gamma::{
    self,
    types::{
        request::{
            MarketByIdRequest, MarketBySlugRequest, MarketTagsRequest, MarketsRequest,
            SearchRequest,
        },
        response::Market,
    },
};

use super::is_numeric_id;
use crate::output::markets::{print_market_detail, print_markets_table};
use crate::output::tags::print_tags_table;
use crate::output::{OutputFormat, print_json};

#[derive(Args)]
pub struct MarketsArgs {
    #[command(subcommand)]
    pub command: MarketsCommand,
}

#[derive(Subcommand)]
pub enum MarketsCommand {
    /// List markets with optional filters
    List {
        /// Filter by active status
        #[arg(long)]
        active: Option<bool>,

        /// Filter by closed status
        #[arg(long)]
        closed: Option<bool>,

        /// Max results
        #[arg(long, default_value = "25")]
        limit: i32,

        /// Pagination offset
        #[arg(long)]
        offset: Option<i32>,

        /// Sort field (e.g. `volume_num`, `liquidity_num`)
        #[arg(long)]
        order: Option<String>,

        /// Sort ascending instead of descending
        #[arg(long)]
        ascending: bool,
    },

    /// Get a single market by ID or slug
    Get {
        /// Market ID (numeric) or slug
        id: String,
    },

    /// Search markets
    Search {
        /// Search query string
        query: String,

        /// Results per type
        #[arg(long, default_value = "10")]
        limit: i32,
    },

    /// Get tags for a market
    Tags {
        /// Market ID
        id: String,
    },
}

pub async fn execute(
    client: &gamma::Client,
    args: MarketsArgs,
    output: OutputFormat,
) -> Result<()> {
    match args.command {
        MarketsCommand::List {
            active,
            closed,
            limit,
            offset,
            order,
            ascending,
        } => {
            let resolved_closed = closed.or_else(|| active.map(|a| !a));

            let request = MarketsRequest::builder()
                .limit(limit)
                .maybe_closed(resolved_closed)
                .maybe_offset(offset)
                .maybe_order(order)
                .maybe_ascending(if ascending { Some(true) } else { None })
                .build();

            let markets = client.markets(&request).await?;

            match output {
                OutputFormat::Table => print_markets_table(&markets),
                OutputFormat::Json => print_json(&markets)?,
            }
        }

        MarketsCommand::Get { id } => {
            let is_numeric = is_numeric_id(&id);
            let market = if is_numeric {
                let req = MarketByIdRequest::builder().id(id).build();
                client.market_by_id(&req).await?
            } else {
                let req = MarketBySlugRequest::builder().slug(id).build();
                client.market_by_slug(&req).await?
            };

            match output {
                OutputFormat::Table => print_market_detail(&market),
                OutputFormat::Json => print_json(&market)?,
            }
        }

        MarketsCommand::Search { query, limit } => {
            let request = SearchRequest::builder()
                .q(query)
                .limit_per_type(limit)
                .build();

            let results = client.search(&request).await?;

            let markets: Vec<Market> = results
                .events
                .unwrap_or_default()
                .into_iter()
                .flat_map(|e| e.markets.unwrap_or_default())
                .collect();

            match output {
                OutputFormat::Table => print_markets_table(&markets),
                OutputFormat::Json => print_json(&markets)?,
            }
        }

        MarketsCommand::Tags { id } => {
            let req = MarketTagsRequest::builder().id(id).build();
            let tags = client.market_tags(&req).await?;

            match output {
                OutputFormat::Table => print_tags_table(&tags),
                OutputFormat::Json => print_json(&tags)?,
            }
        }
    }

    Ok(())
}
