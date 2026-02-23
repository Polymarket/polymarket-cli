pub mod bridge;
pub mod clob;
pub mod comments;
pub mod data;
pub mod events;
pub mod markets;
pub mod profiles;
pub mod series;
pub mod sports;
pub mod tags;

use polymarket_client_sdk::types::Decimal;
use rust_decimal::prelude::ToPrimitive;
use tabled::Table;
use tabled::settings::object::Columns;
use tabled::settings::{Modify, Style, Width};

#[derive(Clone, Debug, clap::ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
}

pub fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let mut truncated: String = s.chars().take(max.saturating_sub(1)).collect();
    truncated.push('\u{2026}');
    truncated
}

pub fn format_decimal(n: Decimal) -> String {
    let f = n.to_f64().unwrap_or(0.0);
    if f >= 1_000_000.0 {
        format!("${:.1}M", f / 1_000_000.0)
    } else if f >= 1_000.0 {
        format!("${:.1}K", f / 1_000.0)
    } else {
        format!("${f:.2}")
    }
}

pub fn print_json(data: &impl serde::Serialize) -> anyhow::Result<()> {
    println!("{}", serde_json::to_string_pretty(data)?);
    Ok(())
}

pub fn print_detail_table(rows: Vec<[String; 2]>) {
    let table = Table::from_iter(rows)
        .with(Style::rounded())
        .with(Modify::new(Columns::first()).with(Width::wrap(20)))
        .with(Modify::new(Columns::last()).with(Width::wrap(80)))
        .to_string();
    println!("{table}");
}

macro_rules! detail_field {
    ($rows:expr, $label:expr, $val:expr) => {
        $rows.push([$label.into(), $val]);
    };
}

pub(crate) use detail_field;
