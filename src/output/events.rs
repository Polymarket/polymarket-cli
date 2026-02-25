use polymarket_client_sdk::gamma::types::response::Event;
use tabled::settings::Style;
use tabled::{Table, Tabled};

use super::{
    NONE, active_status, detail_field, format_date, format_decimal, print_detail_table, truncate,
};

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

fn event_to_row(e: &Event) -> EventRow {
    let title = e.title.as_deref().unwrap_or(NONE);
    let market_count = e
        .markets
        .as_ref()
        .map_or_else(|| NONE.into(), |m| m.len().to_string());

    EventRow {
        title: truncate(title, 60),
        market_count,
        volume: e.volume.map_or_else(|| NONE.into(), format_decimal),
        liquidity: e.liquidity.map_or_else(|| NONE.into(), format_decimal),
        status: active_status(e.closed, e.active).into(),
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
    detail_field!(rows, "Status", active_status(e.closed, e.active).into());
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
        e.start_date.as_ref().map(format_date).unwrap_or_default()
    );
    detail_field!(
        rows,
        "End Date",
        e.end_date.as_ref().map(format_date).unwrap_or_default()
    );
    detail_field!(
        rows,
        "Created At",
        e.created_at.as_ref().map(format_date).unwrap_or_default()
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_event(val: serde_json::Value) -> Event {
        serde_json::from_value(val).unwrap()
    }

    #[test]
    fn status_closed_overrides_active() {
        let e = make_event(json!({"id": "1", "closed": true, "active": true}));
        assert_eq!(active_status(e.closed, e.active), "Closed");
    }

    #[test]
    fn status_active_when_not_closed() {
        let e = make_event(json!({"id": "1", "closed": false, "active": true}));
        assert_eq!(active_status(e.closed, e.active), "Active");
    }

    #[test]
    fn status_inactive_when_fields_missing() {
        let e = make_event(json!({"id": "1"}));
        assert_eq!(active_status(e.closed, e.active), "Inactive");
    }

    #[test]
    fn status_inactive_when_both_false() {
        let e = make_event(json!({"id": "1", "closed": false, "active": false}));
        assert_eq!(active_status(e.closed, e.active), "Inactive");
    }

    #[test]
    fn row_missing_optionals_shows_dashes() {
        let row = event_to_row(&make_event(json!({"id": "1"})));
        assert_eq!(row.title, "—");
        assert_eq!(row.market_count, "—");
        assert_eq!(row.volume, "—");
        assert_eq!(row.liquidity, "—");
        assert_eq!(row.status, "Inactive");
    }

    #[test]
    fn row_counts_markets() {
        let e = make_event(json!({
            "id": "1",
            "markets": [{"id": "m1"}, {"id": "m2"}]
        }));
        assert_eq!(event_to_row(&e).market_count, "2");
    }

    #[test]
    fn row_empty_markets_shows_zero() {
        let e = make_event(json!({"id": "1", "markets": []}));
        assert_eq!(event_to_row(&e).market_count, "0");
    }

    #[test]
    fn row_truncates_long_title() {
        let long_title = "b".repeat(100);
        let e = make_event(json!({"id": "1", "title": long_title}));
        assert_eq!(event_to_row(&e).title.chars().count(), 60);
    }

    #[test]
    fn row_formats_volume() {
        let e = make_event(json!({"id": "1", "volume": "2500000"}));
        assert_eq!(event_to_row(&e).volume, "$2.5M");
    }
}
