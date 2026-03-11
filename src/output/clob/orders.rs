use polymarket_client_sdk::clob::types::response::{
    CancelOrdersResponse, OpenOrderResponse, OrderScoringResponse, OrdersScoringResponse, Page,
    PostOrderResponse, TradeResponse,
};
use serde_json::json;
use tabled::settings::Style;
use tabled::{Table, Tabled};

use super::END_CURSOR;
use crate::output::{OutputFormat, truncate};

pub fn print_orders(result: &Page<OpenOrderResponse>, output: &OutputFormat) -> anyhow::Result<()> {
    match output {
        OutputFormat::Table => {
            if result.data.is_empty() {
                println!("No open orders.");
                return Ok(());
            }
            #[derive(Tabled)]
            struct Row {
                #[tabled(rename = "ID")]
                id: String,
                #[tabled(rename = "Side")]
                side: String,
                #[tabled(rename = "Price")]
                price: String,
                #[tabled(rename = "Size")]
                original_size: String,
                #[tabled(rename = "Matched")]
                size_matched: String,
                #[tabled(rename = "Status")]
                status: String,
                #[tabled(rename = "Type")]
                order_type: String,
            }
            let rows: Vec<Row> = result
                .data
                .iter()
                .map(|o| Row {
                    id: truncate(&o.id, 12),
                    side: o.side.to_string(),
                    price: o.price.to_string(),
                    original_size: o.original_size.to_string(),
                    size_matched: o.size_matched.to_string(),
                    status: o.status.to_string(),
                    order_type: o.order_type.to_string(),
                })
                .collect();
            let table = Table::new(rows).with(Style::rounded()).to_string();
            println!("{table}");
            if result.next_cursor != END_CURSOR {
                println!("Next cursor: {}", result.next_cursor);
            }
        }
        OutputFormat::Json => {
            let data: Vec<_> = result
                .data
                .iter()
                .map(|o| {
                    json!({
                        "id": o.id,
                        "status": o.status.to_string(),
                        "market": o.market.to_string(),
                        "asset_id": o.asset_id.to_string(),
                        "side": o.side.to_string(),
                        "price": o.price.to_string(),
                        "original_size": o.original_size.to_string(),
                        "size_matched": o.size_matched.to_string(),
                        "outcome": o.outcome,
                        "order_type": o.order_type.to_string(),
                        "created_at": o.created_at.to_rfc3339(),
                        "expiration": o.expiration.to_rfc3339(),
                    })
                })
                .collect();
            let wrapper = json!({"data": data, "next_cursor": result.next_cursor});
            crate::output::print_json(&wrapper)?;
        }
    }
    Ok(())
}

pub fn print_order_detail(result: &OpenOrderResponse, output: &OutputFormat) -> anyhow::Result<()> {
    match output {
        OutputFormat::Table => {
            let rows = vec![
                ["ID".into(), result.id.clone()],
                ["Status".into(), result.status.to_string()],
                ["Market".into(), result.market.to_string()],
                ["Asset ID".into(), result.asset_id.to_string()],
                ["Side".into(), result.side.to_string()],
                ["Price".into(), result.price.to_string()],
                ["Original Size".into(), result.original_size.to_string()],
                ["Size Matched".into(), result.size_matched.to_string()],
                ["Outcome".into(), result.outcome.clone()],
                ["Order Type".into(), result.order_type.to_string()],
                ["Created".into(), result.created_at.to_rfc3339()],
                ["Expiration".into(), result.expiration.to_rfc3339()],
                ["Trades".into(), result.associate_trades.join(", ")],
            ];
            crate::output::print_detail_table(rows);
        }
        OutputFormat::Json => {
            let data = json!({
                "id": result.id,
                "status": result.status.to_string(),
                "owner": result.owner.to_string(),
                "maker_address": result.maker_address.to_string(),
                "market": result.market.to_string(),
                "asset_id": result.asset_id.to_string(),
                "side": result.side.to_string(),
                "price": result.price.to_string(),
                "original_size": result.original_size.to_string(),
                "size_matched": result.size_matched.to_string(),
                "outcome": result.outcome,
                "order_type": result.order_type.to_string(),
                "created_at": result.created_at.to_rfc3339(),
                "expiration": result.expiration.to_rfc3339(),
                "associate_trades": result.associate_trades,
            });
            crate::output::print_json(&data)?;
        }
    }
    Ok(())
}

fn post_order_to_json(r: &PostOrderResponse) -> serde_json::Value {
    let tx_hashes: Vec<_> = r
        .transaction_hashes
        .iter()
        .map(std::string::ToString::to_string)
        .collect();
    json!({
        "order_id": r.order_id,
        "status": r.status.to_string(),
        "success": r.success,
        "error_msg": r.error_msg,
        "making_amount": r.making_amount.to_string(),
        "taking_amount": r.taking_amount.to_string(),
        "transaction_hashes": tx_hashes,
        "trade_ids": r.trade_ids,
    })
}

pub fn print_post_order_result(
    result: &PostOrderResponse,
    output: &OutputFormat,
) -> anyhow::Result<()> {
    match output {
        OutputFormat::Table => {
            println!("Order ID: {}", result.order_id);
            println!("Status: {}", result.status);
            println!("Success: {}", result.success);
            if let Some(err) = &result.error_msg
                && !err.is_empty()
            {
                println!("Error: {err}");
            }
            println!("Making: {}", result.making_amount);
            println!("Taking: {}", result.taking_amount);
        }
        OutputFormat::Json => {
            crate::output::print_json(&post_order_to_json(result))?;
        }
    }
    Ok(())
}

pub fn print_post_orders_result(
    results: &[PostOrderResponse],
    output: &OutputFormat,
) -> anyhow::Result<()> {
    match output {
        OutputFormat::Table => {
            for (i, r) in results.iter().enumerate() {
                if i > 0 {
                    println!("---");
                }
                print_post_order_result(r, output)?;
            }
        }
        OutputFormat::Json => {
            let data: Vec<_> = results.iter().map(post_order_to_json).collect();
            crate::output::print_json(&data)?;
        }
    }
    Ok(())
}

pub fn print_cancel_result(
    result: &CancelOrdersResponse,
    output: &OutputFormat,
) -> anyhow::Result<()> {
    match output {
        OutputFormat::Table => {
            if !result.canceled.is_empty() {
                println!("Canceled: {}", result.canceled.join(", "));
            }
            if !result.not_canceled.is_empty() {
                println!("Not canceled:");
                for (id, reason) in &result.not_canceled {
                    println!("  {id}: {reason}");
                }
            }
            if result.canceled.is_empty() && result.not_canceled.is_empty() {
                println!("No orders to cancel.");
            }
        }
        OutputFormat::Json => {
            let data = json!({
                "canceled": result.canceled,
                "not_canceled": result.not_canceled,
            });
            crate::output::print_json(&data)?;
        }
    }
    Ok(())
}

pub fn print_trades(result: &Page<TradeResponse>, output: &OutputFormat) -> anyhow::Result<()> {
    match output {
        OutputFormat::Table => {
            if result.data.is_empty() {
                println!("No trades found.");
                return Ok(());
            }
            #[derive(Tabled)]
            struct Row {
                #[tabled(rename = "ID")]
                id: String,
                #[tabled(rename = "Side")]
                side: String,
                #[tabled(rename = "Price")]
                price: String,
                #[tabled(rename = "Size")]
                size: String,
                #[tabled(rename = "Status")]
                status: String,
                #[tabled(rename = "Time")]
                match_time: String,
            }
            let rows: Vec<Row> = result
                .data
                .iter()
                .map(|t| Row {
                    id: truncate(&t.id, 12),
                    side: t.side.to_string(),
                    price: t.price.to_string(),
                    size: t.size.to_string(),
                    status: t.status.to_string(),
                    match_time: t.match_time.format("%Y-%m-%d %H:%M").to_string(),
                })
                .collect();
            let table = Table::new(rows).with(Style::rounded()).to_string();
            println!("{table}");
            if result.next_cursor != END_CURSOR {
                println!("Next cursor: {}", result.next_cursor);
            }
        }
        OutputFormat::Json => {
            let data: Vec<_> = result
                .data
                .iter()
                .map(|t| {
                    json!({
                        "id": t.id,
                        "taker_order_id": t.taker_order_id,
                        "market": t.market.to_string(),
                        "asset_id": t.asset_id.to_string(),
                        "side": t.side.to_string(),
                        "size": t.size.to_string(),
                        "price": t.price.to_string(),
                        "fee_rate_bps": t.fee_rate_bps.to_string(),
                        "status": t.status.to_string(),
                        "match_time": t.match_time.to_rfc3339(),
                        "outcome": t.outcome,
                        "trader_side": format!("{:?}", t.trader_side),
                        "transaction_hash": t.transaction_hash.to_string(),
                    })
                })
                .collect();
            let wrapper = json!({"data": data, "next_cursor": result.next_cursor});
            crate::output::print_json(&wrapper)?;
        }
    }
    Ok(())
}

pub fn print_order_scoring(
    result: &OrderScoringResponse,
    output: &OutputFormat,
) -> anyhow::Result<()> {
    match output {
        OutputFormat::Table => println!("Scoring: {}", result.scoring),
        OutputFormat::Json => {
            crate::output::print_json(&json!({"scoring": result.scoring}))?;
        }
    }
    Ok(())
}

pub fn print_orders_scoring(
    result: &OrdersScoringResponse,
    output: &OutputFormat,
) -> anyhow::Result<()> {
    match output {
        OutputFormat::Table => {
            if result.is_empty() {
                println!("No scoring data.");
                return Ok(());
            }
            #[derive(Tabled)]
            struct Row {
                #[tabled(rename = "Order ID")]
                order_id: String,
                #[tabled(rename = "Scoring")]
                scoring: String,
            }
            let rows: Vec<Row> = result
                .iter()
                .map(|(id, scoring)| Row {
                    order_id: truncate(id, 16),
                    scoring: scoring.to_string(),
                })
                .collect();
            let table = Table::new(rows).with(Style::rounded()).to_string();
            println!("{table}");
        }
        OutputFormat::Json => {
            crate::output::print_json(result)?;
        }
    }
    Ok(())
}
