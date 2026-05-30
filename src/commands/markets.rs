use anyhow::Result;
use clap::{Args, Subcommand};
use polymarket_client_sdk_v2::gamma::{
    self,
    types::{
        request::{
            MarketByIdRequest, MarketBySlugRequest, MarketTagsRequest, MarketsRequest,
            SearchRequest,
        },
        response::Market,
    },
};

use super::{flag_matches, is_numeric_id};
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

async fn list_markets(
    client: &gamma::Client,
    limit: i32,
    offset: Option<i32>,
    order: Option<String>,
    ascending: bool,
    active: Option<bool>,
    closed: Option<bool>,
) -> Result<Vec<Market>> {
    if limit <= 0 {
        return Ok(Vec::new());
    }
    let page_size = limit;
    let mut next_offset = offset.unwrap_or(0);
    let mut collected: Vec<Market> = Vec::new();

    loop {
        let request = MarketsRequest::builder()
            .limit(page_size)
            .maybe_closed(closed)
            .maybe_offset(Some(next_offset))
            .maybe_order(order.clone())
            .ascending(ascending)
            .build();

        let page = client.markets(&request).await?;
        if page.is_empty() {
            break;
        }

        let raw_count = page.len();
        collected.extend(apply_status_filters(page, active, closed));

        if collected.len() >= page_size as usize {
            collected.truncate(page_size as usize);
            break;
        }

        // Without an active filter, the API-side limit should be authoritative.
        if active.is_none() {
            break;
        }

        // Reached end of available results from the backend.
        if raw_count < page_size as usize {
            break;
        }

        next_offset += raw_count as i32;
    }

    Ok(collected)
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
            let markets =
                list_markets(client, limit, offset, order, ascending, active, closed).await?;
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
