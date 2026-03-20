use anyhow::Result;
use chrono::{DateTime, Utc};
use clap::{Args, Subcommand};
use polymarket_client_sdk::gamma::{
    self,
    types::request::{EventByIdRequest, EventBySlugRequest, EventTagsRequest, EventsRequest},
};
use rust_decimal::Decimal;

use super::is_numeric_id;
use crate::output::OutputFormat;
use crate::output::events::{print_event, print_events};
use crate::output::tags::print_tags;

#[derive(Args)]
pub struct EventsArgs {
    #[command(subcommand)]
    pub command: EventsCommand,
}

#[derive(Subcommand)]
pub enum EventsCommand {
    /// List events with optional filters
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

        /// Sort field (e.g. volume, liquidity, `created_at`)
        #[arg(long)]
        order: Option<String>,

        /// Sort ascending instead of descending
        #[arg(long)]
        ascending: bool,

        /// Filter by tag slug (e.g. "politics", "crypto")
        #[arg(long)]
        tag: Option<String>,

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

        /// Only events starting after this date (e.g. 2026-03-01T00:00:00Z)
        #[arg(long)]
        start_date_min: Option<DateTime<Utc>>,

        /// Only events starting before this date
        #[arg(long)]
        start_date_max: Option<DateTime<Utc>>,

        /// Only events ending after this date
        #[arg(long)]
        end_date_min: Option<DateTime<Utc>>,

        /// Only events ending before this date
        #[arg(long)]
        end_date_max: Option<DateTime<Utc>>,
    },

    /// Get a single event by ID or slug
    Get {
        /// Event ID (numeric) or slug
        id: String,
    },

    /// Get tags for an event
    Tags {
        /// Event ID
        id: String,
    },
}

pub async fn execute(client: &gamma::Client, args: EventsArgs, output: OutputFormat) -> Result<()> {
    match args.command {
        EventsCommand::List {
            active,
            closed,
            limit,
            offset,
            order,
            ascending,
            tag,
            volume_min,
            volume_max,
            liquidity_min,
            liquidity_max,
            start_date_min,
            start_date_max,
            end_date_min,
            end_date_max,
        } => {
            let resolved_closed = closed.or_else(|| active.map(|a| !a));

            let request = EventsRequest::builder()
                .limit(limit)
                .maybe_closed(resolved_closed)
                .maybe_offset(offset)
                .ascending(ascending)
                .maybe_tag_slug(tag)
                // EventsRequest::order is Vec<String>; into_iter on Option yields 0 or 1 items.
                .order(order.into_iter().collect())
                .maybe_volume_min(volume_min)
                .maybe_volume_max(volume_max)
                .maybe_liquidity_min(liquidity_min)
                .maybe_liquidity_max(liquidity_max)
                .maybe_start_date_min(start_date_min)
                .maybe_start_date_max(start_date_max)
                .maybe_end_date_min(end_date_min)
                .maybe_end_date_max(end_date_max)
                .build();

            let events = client.events(&request).await?;
            print_events(&events, &output)?;
        }

        EventsCommand::Get { id } => {
            let is_numeric = is_numeric_id(&id);
            let event = if is_numeric {
                let req = EventByIdRequest::builder().id(id).build();
                client.event_by_id(&req).await?
            } else {
                let req = EventBySlugRequest::builder().slug(id).build();
                client.event_by_slug(&req).await?
            };

            print_event(&event, &output)?;
        }

        EventsCommand::Tags { id } => {
            let req = EventTagsRequest::builder().id(id).build();
            let tags = client.event_tags(&req).await?;

            print_tags(&tags, &output)?;
        }
    }

    Ok(())
}
