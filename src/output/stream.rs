//! Output formatters for WebSocket stream events.
//!
//! Works with the native [`crate::ws::WsEvent`] type — no SDK dependency.

use anyhow::Result;
use serde_json::Value;

use super::OutputFormat;
use crate::ws::WsEvent;

pub fn print_event(event: &WsEvent, output: &OutputFormat) -> Result<()> {
    match output {
        OutputFormat::Json => {
            // Emit the raw payload as a single-line JSON object (NDJSON).
            println!("{}", serde_json::to_string(&event.payload)?);
        }
        OutputFormat::Table => {
            print_table_line(event);
        }
    }
    Ok(())
}

fn print_table_line(event: &WsEvent) {
    let p = &event.payload;
    let asset = truncate_id(&str_field(p, "asset_id"));

    match event.event_type.as_str() {
        "book" => {
            let bids = p.get("bids").and_then(Value::as_array).map_or(0, Vec::len);
            let asks = p.get("asks").and_then(Value::as_array).map_or(0, Vec::len);
            let best_bid = p
                .get("bids")
                .and_then(Value::as_array)
                .and_then(|a| a.first())
                .and_then(|l| l.get("price"))
                .map_or_else(|| "-".into(), |v| v.to_string());
            let best_ask = p
                .get("asks")
                .and_then(Value::as_array)
                .and_then(|a| a.first())
                .and_then(|l| l.get("price"))
                .map_or_else(|| "-".into(), |v| v.to_string());
            println!("BOOK  | Asset: {asset} | Bid: {best_bid:<6} | Ask: {best_ask:<6} | Levels: {bids}/{asks}");
        }
        "price_change" => {
            if let Some(changes) = p.get("price_changes").and_then(Value::as_array) {
                for pc in changes {
                    let id = truncate_id(&str_field(pc, "asset_id"));
                    let price = str_field(pc, "price");
                    let side = str_field(pc, "side").to_uppercase();
                    println!("PRICE | Asset: {id} | Price: {price:<6} | Side: {side:<4}");
                }
            }
        }
        "last_trade_price" => {
            let price = str_field(p, "price");
            println!("TRADE | Asset: {asset} | Price: {price:<6}");
        }
        "midpoint" => {
            let mid = str_field(p, "midpoint");
            println!("MID   | Asset: {asset} | Price: {mid:<6}");
        }
        other => {
            let prefix = other.to_uppercase();
            let safe_prefix: String = prefix.chars().take(5).collect();
            println!("{safe_prefix:<5} | {}", serde_json::to_string(p).unwrap_or_default());
        }
    }
}

fn str_field(v: &Value, key: &str) -> String {
    v.get(key)
        .map_or_else(|| "-".into(), |val| val.as_str().unwrap_or(&val.to_string()).to_string())
}

fn truncate_id(s: &str) -> String {
    if s.chars().count() <= 12 {
        s.to_string()
    } else {
        let prefix: String = s.chars().take(6).collect();
        let suffix: String = s.chars().rev().take(4).collect::<Vec<_>>().into_iter().rev().collect();
        format!("{prefix}…{suffix}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_short_id_unchanged() {
        assert_eq!(truncate_id("12345"), "12345");
    }

    #[test]
    fn truncate_long_id_abbreviated() {
        let long_id = "106585164761922456203746651621390029417453862034640469075081961934906147433548";
        let result = truncate_id(long_id);
        assert!(result.contains('…'), "Expected ellipsis in: {result}");
        assert!(result.starts_with("106585"));
        assert!(result.ends_with("3548"));
    }
}
