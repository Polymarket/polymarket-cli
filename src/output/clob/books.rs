use polymarket_client_sdk_v2::clob::types::response::{
    LastTradePriceResponse, LastTradesPricesResponse, OrderBookSummaryResponse, OrderSummary,
};
use serde_json::json;
use tabled::settings::Style;
use tabled::{Table, Tabled};

use crate::output::{DASH, OutputFormat, truncate};

/// Returns bids in display order (best bid first).
///
/// The CLOB API returns bids ascending by price, so the best (highest-priced)
/// bid is reversed to the top for human-facing display.
fn bids_in_display_order(bids: &[OrderSummary]) -> Vec<&OrderSummary> {
    bids.iter().rev().collect()
}

/// Returns asks in display order (best ask first).
///
/// The CLOB API returns asks descending by price, so the best (lowest-priced)
/// ask is reversed to the top for human-facing display.
fn asks_in_display_order(asks: &[OrderSummary]) -> Vec<&OrderSummary> {
    asks.iter().rev().collect()
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
                    .map_or(DASH.into(), |p| p.to_string())
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
                let rows: Vec<Row> = bids_in_display_order(&result.bids)
                    .into_iter()
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
                let rows: Vec<Row> = asks_in_display_order(&result.asks)
                    .into_iter()
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
            crate::output::print_json(result)?;
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
            crate::output::print_json(result)?;
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

#[cfg(test)]
mod tests {
    use super::{OrderSummary, asks_in_display_order, bids_in_display_order};
    use rust_decimal_macros::dec;

    fn order(price: rust_decimal::Decimal) -> OrderSummary {
        OrderSummary::builder().price(price).size(dec!(100)).build()
    }

    // The CLOB API returns bids ascending (worst price first).
    // The display order helper must reverse that so the best (highest) bid is first.
    #[test]
    fn bids_in_display_order_puts_best_bid_first() {
        let api_order = vec![order(dec!(0.30)), order(dec!(0.40)), order(dec!(0.50))];

        let displayed: Vec<rust_decimal::Decimal> = bids_in_display_order(&api_order)
            .into_iter()
            .map(|o| o.price)
            .collect();

        assert_eq!(
            displayed,
            vec![dec!(0.50), dec!(0.40), dec!(0.30)],
            "bids must be reversed for display so the highest-priced bid is on top"
        );
    }

    // The CLOB API returns asks descending (worst price first).
    // The display order helper must reverse that so the best (lowest) ask is first.
    #[test]
    fn asks_in_display_order_puts_best_ask_first() {
        let api_order = vec![order(dec!(0.70)), order(dec!(0.60)), order(dec!(0.50))];

        let displayed: Vec<rust_decimal::Decimal> = asks_in_display_order(&api_order)
            .into_iter()
            .map(|o| o.price)
            .collect();

        assert_eq!(
            displayed,
            vec![dec!(0.50), dec!(0.60), dec!(0.70)],
            "asks must be reversed for display so the lowest-priced ask is on top"
        );
    }

    // Empty input must not panic and must produce an empty display order.
    #[test]
    fn empty_inputs_produce_empty_output() {
        let empty: Vec<OrderSummary> = vec![];
        assert!(bids_in_display_order(&empty).is_empty());
        assert!(asks_in_display_order(&empty).is_empty());
    }
}
