use polymarket_client_sdk::gamma::types::response::Series;
use tabled::settings::Style;
use tabled::{Table, Tabled};

use super::{detail_field, format_decimal, print_detail_table, truncate};

#[derive(Tabled)]
struct SeriesRow {
    #[tabled(rename = "Title")]
    title: String,
    #[tabled(rename = "Type")]
    series_type: String,
    #[tabled(rename = "Volume")]
    volume: String,
    #[tabled(rename = "Liquidity")]
    liquidity: String,
    #[tabled(rename = "Status")]
    status: String,
}

fn series_status(s: &Series) -> &'static str {
    if s.closed == Some(true) {
        "Closed"
    } else if s.active == Some(true) {
        "Active"
    } else {
        "Inactive"
    }
}

fn series_to_row(s: &Series) -> SeriesRow {
    SeriesRow {
        title: truncate(s.title.as_deref().unwrap_or("—"), 50),
        series_type: s.series_type.as_deref().unwrap_or("—").into(),
        volume: s.volume.map_or_else(|| "—".into(), format_decimal),
        liquidity: s.liquidity.map_or_else(|| "—".into(), format_decimal),
        status: series_status(s).into(),
    }
}

pub fn print_series_table(series: &[Series]) {
    if series.is_empty() {
        println!("No series found.");
        return;
    }
    let rows: Vec<SeriesRow> = series.iter().map(series_to_row).collect();
    let table = Table::new(rows).with(Style::rounded()).to_string();
    println!("{table}");
}

pub fn print_series_detail(s: &Series) {
    let mut rows: Vec<[String; 2]> = Vec::new();

    detail_field!(rows, "ID", s.id.clone());
    detail_field!(rows, "Title", s.title.clone().unwrap_or_default());
    detail_field!(rows, "Slug", s.slug.clone().unwrap_or_default());
    detail_field!(rows, "Type", s.series_type.clone().unwrap_or_default());
    detail_field!(rows, "Recurrence", s.recurrence.clone().unwrap_or_default());
    detail_field!(
        rows,
        "Description",
        s.description.clone().unwrap_or_default()
    );
    detail_field!(
        rows,
        "Volume",
        s.volume.map(format_decimal).unwrap_or_default()
    );
    detail_field!(
        rows,
        "Liquidity",
        s.liquidity.map(format_decimal).unwrap_or_default()
    );
    detail_field!(
        rows,
        "Volume (24hr)",
        s.volume_24hr.map(format_decimal).unwrap_or_default()
    );
    detail_field!(rows, "Status", series_status(s).into());
    detail_field!(
        rows,
        "Events",
        s.events
            .as_ref()
            .map(|e| e.len().to_string())
            .unwrap_or_default()
    );
    detail_field!(
        rows,
        "Comment Count",
        s.comment_count.map(|c| c.to_string()).unwrap_or_default()
    );
    detail_field!(
        rows,
        "Start Date",
        s.start_date.map(|d| d.to_string()).unwrap_or_default()
    );
    detail_field!(
        rows,
        "Created At",
        s.created_at.map(|d| d.to_string()).unwrap_or_default()
    );
    detail_field!(
        rows,
        "Tags",
        s.tags
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
