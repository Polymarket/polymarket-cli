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

fn apply_status_filters(
    markets: Vec<Market>,
    active_filter: Option<bool>,
    closed_filter: Option<bool>,
) -> Vec<Market> {
    markets
        .into_iter()
        .filter(|market| {
            flag_matches(market.active, active_filter) && flag_matches(market.closed, closed_filter)
        })
        .collect()
}

fn flag_matches(value: Option<bool>, filter: Option<bool>) -> bool {
    filter.is_none_or(|expected| value == Some(expected))
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
            let request = MarketsRequest::builder()
                .limit(limit)
                .maybe_closed(closed)
                .maybe_offset(offset)
                .maybe_order(order)
                .maybe_ascending(if ascending { Some(true) } else { None })
                .build();

            let markets = apply_status_filters(client.markets(&request).await?, active, closed);

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

#[cfg(test)]
mod tests {
    use super::apply_status_filters;
    use polymarket_client_sdk::gamma::types::response::Market;
    use serde_json::json;

    fn make_market(value: serde_json::Value) -> Market {
        serde_json::from_value(value).unwrap()
    }

    #[test]
    fn status_filters_are_independent() {
        let markets = vec![
            make_market(json!({"id":"1", "active": true, "closed": true})),
            make_market(json!({"id":"2", "active": false, "closed": true})),
            make_market(json!({"id":"3", "active": false, "closed": false})),
        ];

        let filtered = apply_status_filters(markets, Some(false), Some(true));

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "2");
    }

    #[test]
    fn active_filter_does_not_imply_closed_filter() {
        let markets = vec![
            make_market(json!({"id":"1", "active": false, "closed": true})),
            make_market(json!({"id":"2", "active": false, "closed": false})),
        ];

        let filtered = apply_status_filters(markets, Some(false), None);

        assert_eq!(filtered.len(), 2);
    }
}
