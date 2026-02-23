use polymarket_client_sdk::gamma::types::response::Event;
use tabled::settings::Style;
use tabled::{Table, Tabled};

use super::{detail_field, format_decimal, print_detail_table, truncate};

#[derive(Tabled)]
struct EventRow {
    #[tabled(rename = "Title")]
    title: String,
    #[tabled(rename = "Markets")]
    market_count: String,
    #[tabled(rename = "Volume")]
    volume: String,
    #[tabled(rename = "Liquidity")]
    liquidity: String,
    #[tabled(rename = "Status")]
    status: String,
}

fn event_status(e: &Event) -> &'static str {
    if e.closed == Some(true) {
        "Closed"
    } else if e.active == Some(true) {
        "Active"
    } else {
        "Inactive"
    }
}

fn event_to_row(e: &Event) -> EventRow {
    let title = e.title.as_deref().unwrap_or("—");
    let market_count = e
        .markets
        .as_ref()
        .map_or_else(|| "—".into(), |m| m.len().to_string());

    EventRow {
        title: truncate(title, 60),
        market_count,
        volume: e.volume.map_or_else(|| "—".into(), format_decimal),
        liquidity: e.liquidity.map_or_else(|| "—".into(), format_decimal),
        status: event_status(e).into(),
    }
}

pub fn print_events_table(events: &[Event]) {
    if events.is_empty() {
        println!("No events found.");
        return;
    }
    let rows: Vec<EventRow> = events.iter().map(event_to_row).collect();
    let table = Table::new(rows).with(Style::rounded()).to_string();
    println!("{table}");
}

#[allow(clippy::too_many_lines)]
pub fn print_event_detail(e: &Event) {
    let mut rows: Vec<[String; 2]> = Vec::new();

    detail_field!(rows, "ID", e.id.clone());
    detail_field!(rows, "Title", e.title.clone().unwrap_or_default());
    detail_field!(rows, "Slug", e.slug.clone().unwrap_or_default());
    detail_field!(
        rows,
        "Description",
        e.description.clone().unwrap_or_default()
    );
    detail_field!(rows, "Category", e.category.clone().unwrap_or_default());
    detail_field!(
        rows,
        "Markets",
        e.markets
            .as_ref()
            .map(|m| {
                if m.is_empty() {
                    "None".into()
                } else {
                    m.iter()
                        .filter_map(|mkt| mkt.question.as_deref())
                        .collect::<Vec<_>>()
                        .join(" | ")
                }
            })
            .unwrap_or_default()
    );
    detail_field!(
        rows,
        "Volume",
        e.volume.map(format_decimal).unwrap_or_default()
    );
    detail_field!(
        rows,
        "Liquidity",
        e.liquidity.map(format_decimal).unwrap_or_default()
    );
    detail_field!(
        rows,
        "Open Interest",
        e.open_interest.map(format_decimal).unwrap_or_default()
    );
    detail_field!(
        rows,
        "Volume (24hr)",
        e.volume_24hr.map(format_decimal).unwrap_or_default()
    );
    detail_field!(
        rows,
        "Volume (1wk)",
        e.volume_1wk.map(format_decimal).unwrap_or_default()
    );
    detail_field!(
        rows,
        "Volume (1mo)",
        e.volume_1mo.map(format_decimal).unwrap_or_default()
    );
    detail_field!(rows, "Status", event_status(e).into());
    detail_field!(
        rows,
        "Neg Risk",
        e.neg_risk.map(|v| v.to_string()).unwrap_or_default()
    );
    detail_field!(
        rows,
        "Neg Risk Market ID",
        e.neg_risk_market_id
            .map(|id| format!("{id}"))
            .unwrap_or_default()
    );
    detail_field!(
        rows,
        "Comment Count",
        e.comment_count.map(|c| c.to_string()).unwrap_or_default()
    );
    detail_field!(
        rows,
        "Start Date",
        e.start_date.map(|d| d.to_string()).unwrap_or_default()
    );
    detail_field!(
        rows,
        "End Date",
        e.end_date.map(|d| d.to_string()).unwrap_or_default()
    );
    detail_field!(
        rows,
        "Created At",
        e.created_at.map(|d| d.to_string()).unwrap_or_default()
    );
    detail_field!(
        rows,
        "Resolution Source",
        e.resolution_source.clone().unwrap_or_default()
    );
    detail_field!(
        rows,
        "Tags",
        e.tags
            .as_ref()
            .map(|tags| {
                tags.iter()
                    .filter_map(|t| t.label.as_deref())
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default()
    );

    print_detail_table(rows);
}
