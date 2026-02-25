use polymarket_client_sdk::clob::types::response::{
    LastTradePriceResponse, LastTradesPricesResponse, OrderBookSummaryResponse,
};
use serde_json::json;
use tabled::settings::Style;
use tabled::{Table, Tabled};

use crate::output::{OutputFormat, truncate};

fn order_book_to_json(book: &OrderBookSummaryResponse) -> serde_json::Value {
    let bids: Vec<_> = book
        .bids
        .iter()
        .map(|o| json!({"price": o.price.to_string(), "size": o.size.to_string()}))
        .collect();
    let asks: Vec<_> = book
        .asks
        .iter()
        .map(|o| json!({"price": o.price.to_string(), "size": o.size.to_string()}))
        .collect();
    json!({
        "market": book.market.to_string(),
        "asset_id": book.asset_id.to_string(),
        "timestamp": book.timestamp.to_rfc3339(),
        "bids": bids,
        "asks": asks,
        "min_order_size": book.min_order_size.to_string(),
        "neg_risk": book.neg_risk,
        "tick_size": book.tick_size.as_decimal().to_string(),
        "last_trade_price": book.last_trade_price.map(|p| p.to_string()),
    })
}

pub fn print_order_book(
    result: &OrderBookSummaryResponse,
    output: &OutputFormat,
) -> anyhow::Result<()> {
    match output {
        OutputFormat::Table => {
            println!("Market: {}", result.market);
            println!("Asset: {}", result.asset_id);
            println!(
                "Last Trade: {}",
                result
                    .last_trade_price
                    .map_or("—".into(), |p| p.to_string())
            );
            println!();

            #[derive(Tabled)]
            struct Row {
                #[tabled(rename = "Price")]
                price: String,
                #[tabled(rename = "Size")]
                size: String,
            }

            if result.bids.is_empty() {
                println!("No bids.");
            } else {
                println!("Bids:");
                let rows: Vec<Row> = result
                    .bids
                    .iter()
                    .map(|o| Row {
                        price: o.price.to_string(),
                        size: o.size.to_string(),
                    })
                    .collect();
                let table = Table::new(rows).with(Style::rounded()).to_string();
                println!("{table}");
            }

            println!();

            if result.asks.is_empty() {
                println!("No asks.");
            } else {
                println!("Asks:");
                let rows: Vec<Row> = result
                    .asks
                    .iter()
                    .map(|o| Row {
                        price: o.price.to_string(),
                        size: o.size.to_string(),
                    })
                    .collect();
                let table = Table::new(rows).with(Style::rounded()).to_string();
                println!("{table}");
            }
        }
        OutputFormat::Json => {
            crate::output::print_json(&order_book_to_json(result))?;
        }
    }
    Ok(())
}

pub fn print_order_books(
    result: &[OrderBookSummaryResponse],
    output: &OutputFormat,
) -> anyhow::Result<()> {
    match output {
        OutputFormat::Table => {
            if result.is_empty() {
                println!("No order books found.");
                return Ok(());
            }
            for (i, book) in result.iter().enumerate() {
                if i > 0 {
                    println!();
                }
                print_order_book(book, output)?;
            }
        }
        OutputFormat::Json => {
            let data: Vec<_> = result.iter().map(order_book_to_json).collect();
            crate::output::print_json(&data)?;
        }
    }
    Ok(())
}

pub fn print_last_trade(
    result: &LastTradePriceResponse,
    output: &OutputFormat,
) -> anyhow::Result<()> {
    match output {
        OutputFormat::Table => println!("Last Trade: {} ({})", result.price, result.side),
        OutputFormat::Json => {
            crate::output::print_json(&json!({
                "price": result.price.to_string(),
                "side": result.side.to_string(),
            }))?;
        }
    }
    Ok(())
}

pub fn print_last_trades_prices(
    result: &[LastTradesPricesResponse],
    output: &OutputFormat,
) -> anyhow::Result<()> {
    match output {
        OutputFormat::Table => {
            if result.is_empty() {
                println!("No last trade prices found.");
                return Ok(());
            }
            #[derive(Tabled)]
            struct Row {
                #[tabled(rename = "Token ID")]
                token_id: String,
                #[tabled(rename = "Price")]
                price: String,
                #[tabled(rename = "Side")]
                side: String,
            }
            let rows: Vec<Row> = result
                .iter()
                .map(|t| Row {
                    token_id: truncate(&t.token_id.to_string(), 20),
                    price: t.price.to_string(),
                    side: t.side.to_string(),
                })
                .collect();
            let table = Table::new(rows).with(Style::rounded()).to_string();
            println!("{table}");
        }
        OutputFormat::Json => {
            let data: Vec<_> = result
                .iter()
                .map(|t| {
                    json!({
                        "token_id": t.token_id.to_string(),
                        "price": t.price.to_string(),
                        "side": t.side.to_string(),
                    })
                })
                .collect();
            crate::output::print_json(&data)?;
        }
    }
    Ok(())
}
