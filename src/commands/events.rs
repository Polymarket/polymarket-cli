use anyhow::Result;
use clap::{Args, Subcommand};
use polymarket_client_sdk_v2::gamma::{
    self,
    types::request::{EventByIdRequest, EventBySlugRequest, EventTagsRequest, EventsRequest},
};

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
        } => {
            let request =
                build_events_list_request(active, closed, limit, offset, order, ascending, tag);
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

fn build_events_list_request(
    active: Option<bool>,
    closed: Option<bool>,
    limit: i32,
    offset: Option<i32>,
    order: Option<String>,
    ascending: bool,
    tag: Option<String>,
) -> EventsRequest {
    let resolved_closed = Some(closed.unwrap_or_else(|| active.map(|a| !a).unwrap_or(false)));

    EventsRequest::builder()
        .limit(limit)
        .maybe_closed(resolved_closed)
        .maybe_offset(offset)
        .ascending(ascending)
        .maybe_tag_slug(tag)
        // EventsRequest::order is Vec<String>; into_iter on Option yields 0 or 1 items.
        .order(order.into_iter().collect())
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn events_list_resolves_closed_filter() {
        assert_eq!(
            build_events_list_request(None, None, 25, None, None, false, None).closed,
            Some(false)
        );
        assert_eq!(
            build_events_list_request(None, Some(true), 25, None, None, false, None).closed,
            Some(true)
        );
        assert_eq!(
            build_events_list_request(Some(true), None, 25, None, None, false, None).closed,
            Some(false)
        );
        assert_eq!(
            build_events_list_request(Some(false), None, 25, None, None, false, None).closed,
            Some(true)
        );
        assert_eq!(
            build_events_list_request(Some(true), Some(true), 25, None, None, false, None).closed,
            Some(true)
        );
    }
}
