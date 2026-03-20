use anyhow::Result;
use chrono::{DateTime, Utc};
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
use rust_decimal::Decimal;

use super::is_numeric_id;
use crate::output::OutputFormat;
use crate::output::markets::{print_market, print_markets};
use crate::output::tags::print_tags;

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

        /// Minimum trading volume (e.g. 1000000)
        #[arg(long)]
        volume_min: Option<Decimal>,

        /// Maximum trading volume
        #[arg(long)]
        volume_max: Option<Decimal>,

        /// Minimum liquidity
        #[arg(long)]
        liquidity_min: Option<Decimal>,

        /// Maximum liquidity
        #[arg(long)]
        liquidity_max: Option<Decimal>,

        /// Only markets starting after this date (e.g. 2026-03-01T00:00:00Z)
        #[arg(long)]
        start_date_min: Option<DateTime<Utc>>,

        /// Only markets starting before this date
        #[arg(long)]
        start_date_max: Option<DateTime<Utc>>,

        /// Only markets ending after this date
        #[arg(long)]
        end_date_min: Option<DateTime<Utc>>,

        /// Only markets ending before this date
        #[arg(long)]
        end_date_max: Option<DateTime<Utc>>,

        /// Filter by tag ID
        #[arg(long)]
        tag: Option<String>,
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
            volume_min,
            volume_max,
            liquidity_min,
            liquidity_max,
            start_date_min,
            start_date_max,
            end_date_min,
            end_date_max,
            tag,
        } => {
            let resolved_closed = closed.or_else(|| active.map(|a| !a));

            let request = MarketsRequest::builder()
                .limit(limit)
                .maybe_closed(resolved_closed)
                .maybe_offset(offset)
                .maybe_order(order)
                .ascending(ascending)
                .maybe_volume_num_min(volume_min)
                .maybe_volume_num_max(volume_max)
                .maybe_liquidity_num_min(liquidity_min)
                .maybe_liquidity_num_max(liquidity_max)
                .maybe_start_date_min(start_date_min)
                .maybe_start_date_max(start_date_max)
                .maybe_end_date_min(end_date_min)
                .maybe_end_date_max(end_date_max)
                .maybe_tag_id(tag)
                .build();

            let markets = client.markets(&request).await?;
            print_markets(&markets, &output)?;
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

            print_market(&market, &output)?;
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

            print_markets(&markets, &output)?;
        }

        MarketsCommand::Tags { id } => {
            let req = MarketTagsRequest::builder().id(id).build();
            let tags = client.market_tags(&req).await?;

            print_tags(&tags, &output)?;
        }
    }

    Ok(())
}
