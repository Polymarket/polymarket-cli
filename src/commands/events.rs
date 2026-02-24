use anyhow::Result;
use clap::{Args, Subcommand};
use polymarket_client_sdk::gamma::{
    self,
    types::{
        request::{EventByIdRequest, EventBySlugRequest, EventTagsRequest, EventsRequest},
        response::Event,
    },
};

use super::{flag_matches, is_numeric_id};
use crate::output::events::{print_event_detail, print_events_table};
use crate::output::tags::print_tags_table;
use crate::output::{OutputFormat, print_json};

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

fn apply_status_filters(
    events: Vec<Event>,
    active_filter: Option<bool>,
    closed_filter: Option<bool>,
) -> Vec<Event> {
    events
        .into_iter()
        .filter(|event| {
            flag_matches(event.active, active_filter) && flag_matches(event.closed, closed_filter)
        })
        .collect()
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
            let request = EventsRequest::builder()
                .limit(limit)
                .maybe_active(active)
                .maybe_closed(closed)
                .maybe_offset(offset)
                .maybe_ascending(if ascending { Some(true) } else { None })
                .maybe_tag_slug(tag)
                .order(order.into_iter().collect::<Vec<_>>())
                .build();

            let events = apply_status_filters(client.events(&request).await?, active, closed);

            match output {
                OutputFormat::Table => print_events_table(&events),
                OutputFormat::Json => print_json(&events)?,
            }
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

            match output {
                OutputFormat::Table => print_event_detail(&event),
                OutputFormat::Json => print_json(&event)?,
            }
        }

        EventsCommand::Tags { id } => {
            let req = EventTagsRequest::builder().id(id).build();
            let tags = client.event_tags(&req).await?;

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
    use polymarket_client_sdk::gamma::types::response::Event;
    use serde_json::json;

    fn make_event(value: serde_json::Value) -> Event {
        serde_json::from_value(value).unwrap()
    }

    #[test]
    fn status_filters_are_independent() {
        let events = vec![
            make_event(json!({"id":"1", "active": true, "closed": true})),
            make_event(json!({"id":"2", "active": false, "closed": true})),
            make_event(json!({"id":"3", "active": false, "closed": false})),
        ];

        let filtered = apply_status_filters(events, Some(false), Some(true));

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "2");
    }

    #[test]
    fn active_filter_does_not_imply_closed_filter() {
        let events = vec![
            make_event(json!({"id":"1", "active": false, "closed": true})),
            make_event(json!({"id":"2", "active": false, "closed": false})),
        ];

        let filtered = apply_status_filters(events, Some(false), None);

        assert_eq!(filtered.len(), 2);
    }
}
