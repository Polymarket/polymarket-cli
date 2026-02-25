pub mod approve;
pub mod bridge;
pub mod clob;
pub mod comments;
pub mod ctf;
pub mod data;
pub mod events;
pub mod markets;
pub mod profiles;
pub mod series;
pub mod sports;
pub mod tags;

use chrono::{DateTime, Utc};
use polymarket_client_sdk::types::Decimal;
use rust_decimal::prelude::ToPrimitive;
use tabled::Table;
use tabled::settings::object::Columns;
use tabled::settings::{Modify, Style, Width};

/// Display string for missing/null values in table output.
pub const NONE: &str = "—";

#[derive(Clone, Copy, Debug, clap::ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
}

pub(crate) fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let mut truncated: String = s.chars().take(max.saturating_sub(1)).collect();
    truncated.push('\u{2026}');
    truncated
}

pub(crate) fn format_decimal(n: Decimal) -> String {
    let f = n.to_f64().unwrap_or(0.0);
    if f >= 1_000_000.0 {
        format!("${:.1}M", f / 1_000_000.0)
    } else if f >= 1_000.0 {
        format!("${:.1}K", f / 1_000.0)
    } else {
        format!("${f:.2}")
    }
}

pub(crate) fn format_date(d: &DateTime<Utc>) -> String {
    d.format("%Y-%m-%d %H:%M UTC").to_string()
}

pub(crate) fn active_status(closed: Option<bool>, active: Option<bool>) -> &'static str {
    if closed == Some(true) {
        "Closed"
    } else if active == Some(true) {
        "Active"
    } else {
        "Inactive"
    }
}

pub(crate) fn print_json(data: &impl serde::Serialize) -> anyhow::Result<()> {
    println!("{}", serde_json::to_string_pretty(data)?);
    Ok(())
}

/// Print an error in the appropriate format for the current output mode.
pub(crate) fn print_error(error: &anyhow::Error, format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::json!({"error": error.to_string()}));
        }
        OutputFormat::Table => {
            eprintln!("Error: {error}");
        }
    }
}

pub(crate) fn print_detail_table(rows: Vec<[String; 2]>) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn truncate_shorter_than_max_unchanged() {
        assert_eq!(truncate("hello", 10), "hello");
    }

    #[test]
    fn truncate_exact_length_unchanged() {
        assert_eq!(truncate("hello", 5), "hello");
    }

    #[test]
    fn truncate_over_max_appends_ellipsis() {
        assert_eq!(truncate("hello world", 6), "hello\u{2026}");
    }

    #[test]
    fn truncate_max_one_is_just_ellipsis() {
        assert_eq!(truncate("hello", 1), "\u{2026}");
    }

    #[test]
    fn truncate_max_zero_is_just_ellipsis() {
        assert_eq!(truncate("hello", 0), "\u{2026}");
    }

    #[test]
    fn truncate_empty_string_unchanged() {
        assert_eq!(truncate("", 5), "");
    }

    #[test]
    fn truncate_counts_chars_not_bytes() {
        // "café!" is 5 chars but 6 bytes (é is 2 bytes)
        assert_eq!(truncate("café!", 3), "ca\u{2026}");
    }

    #[test]
    fn format_decimal_millions() {
        assert_eq!(format_decimal(dec!(1_500_000)), "$1.5M");
    }

    #[test]
    fn format_decimal_at_million_boundary() {
        assert_eq!(format_decimal(dec!(1_000_000)), "$1.0M");
    }

    #[test]
    fn format_decimal_thousands() {
        assert_eq!(format_decimal(dec!(1_500)), "$1.5K");
    }

    #[test]
    fn format_decimal_at_thousand_boundary() {
        assert_eq!(format_decimal(dec!(1_000)), "$1.0K");
    }

    #[test]
    fn format_decimal_just_below_thousand() {
        assert_eq!(format_decimal(dec!(999)), "$999.00");
    }

    #[test]
    fn format_decimal_sub_dollar() {
        assert_eq!(format_decimal(dec!(0.5)), "$0.50");
    }

    #[test]
    fn format_decimal_zero() {
        assert_eq!(format_decimal(dec!(0)), "$0.00");
    }

    #[test]
    fn format_decimal_negative() {
        assert_eq!(format_decimal(dec!(-500)), "$-500.00");
    }

    #[test]
    fn format_decimal_just_below_million_uses_k() {
        assert_eq!(format_decimal(dec!(999_999)), "$1000.0K");
    }
}
