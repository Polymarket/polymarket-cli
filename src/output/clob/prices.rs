use polymarket_client_sdk::clob::types::response::{
    MidpointResponse, MidpointsResponse, PriceResponse, PricesResponse, SpreadResponse,
    SpreadsResponse,
};
use serde_json::json;
use tabled::settings::Style;
use tabled::{Table, Tabled};

use crate::output::{OutputFormat, truncate};

pub fn print_price(result: &PriceResponse, output: &OutputFormat) -> anyhow::Result<()> {
    match output {
        OutputFormat::Table => println!("Price: {}", result.price),
        OutputFormat::Json => {
            crate::output::print_json(&json!({"price": result.price.to_string()}))?;
        }
    }
    Ok(())
}

pub fn print_batch_prices(result: &PricesResponse, output: &OutputFormat) -> anyhow::Result<()> {
    match output {
        OutputFormat::Table => {
            let Some(prices) = &result.prices else {
                println!("No prices available.");
                return Ok(());
            };
            if prices.is_empty() {
                println!("No prices available.");
                return Ok(());
            }
            #[derive(Tabled)]
            struct Row {
                #[tabled(rename = "Token ID")]
                token_id: String,
                #[tabled(rename = "Side")]
                side: String,
                #[tabled(rename = "Price")]
                price: String,
            }
            let mut rows = Vec::new();
            for (token_id, sides) in prices {
                for (side, price) in sides {
                    rows.push(Row {
                        token_id: truncate(&token_id.to_string(), 20),
                        side: side.to_string(),
                        price: price.to_string(),
                    });
                }
            }
            let table = Table::new(rows).with(Style::rounded()).to_string();
            println!("{table}");
        }
        OutputFormat::Json => {
            let data = result.prices.as_ref().map(|prices| {
                prices
                    .iter()
                    .map(|(token_id, sides)| {
                        let side_map: serde_json::Map<String, serde_json::Value> = sides
                            .iter()
                            .map(|(side, price)| (side.to_string(), json!(price.to_string())))
                            .collect();
                        (token_id.to_string(), json!(side_map))
                    })
                    .collect::<serde_json::Map<String, serde_json::Value>>()
            });
            crate::output::print_json(&data)?;
        }
    }
    Ok(())
}

pub fn print_midpoint(result: &MidpointResponse, output: &OutputFormat) -> anyhow::Result<()> {
    match output {
        OutputFormat::Table => println!("Midpoint: {}", result.mid),
        OutputFormat::Json => {
            crate::output::print_json(&json!({"midpoint": result.mid.to_string()}))?;
        }
    }
    Ok(())
}

pub fn print_midpoints(result: &MidpointsResponse, output: &OutputFormat) -> anyhow::Result<()> {
    match output {
        OutputFormat::Table => {
            if result.midpoints.is_empty() {
                println!("No midpoints available.");
                return Ok(());
            }
            #[derive(Tabled)]
            struct Row {
                #[tabled(rename = "Token ID")]
                token_id: String,
                #[tabled(rename = "Midpoint")]
                midpoint: String,
            }
            let rows: Vec<Row> = result
                .midpoints
                .iter()
                .map(|(id, mid)| Row {
                    token_id: truncate(&id.to_string(), 20),
                    midpoint: mid.to_string(),
                })
                .collect();
            let table = Table::new(rows).with(Style::rounded()).to_string();
            println!("{table}");
        }
        OutputFormat::Json => {
            let data: serde_json::Map<String, serde_json::Value> = result
                .midpoints
                .iter()
                .map(|(id, mid)| (id.to_string(), json!(mid.to_string())))
                .collect();
            crate::output::print_json(&data)?;
        }
    }
    Ok(())
}

pub fn print_spread(result: &SpreadResponse, output: &OutputFormat) -> anyhow::Result<()> {
    match output {
        OutputFormat::Table => println!("Spread: {}", result.spread),
        OutputFormat::Json => {
            crate::output::print_json(&json!({"spread": result.spread.to_string()}))?;
        }
    }
    Ok(())
}

pub fn print_spreads(result: &SpreadsResponse, output: &OutputFormat) -> anyhow::Result<()> {
    match output {
        OutputFormat::Table => {
            let Some(spreads) = &result.spreads else {
                println!("No spreads available.");
                return Ok(());
            };
            if spreads.is_empty() {
                println!("No spreads available.");
                return Ok(());
            }
            #[derive(Tabled)]
            struct Row {
                #[tabled(rename = "Token ID")]
                token_id: String,
                #[tabled(rename = "Spread")]
                spread: String,
            }
            let rows: Vec<Row> = spreads
                .iter()
                .map(|(id, spread)| Row {
                    token_id: truncate(&id.to_string(), 20),
                    spread: spread.to_string(),
                })
                .collect();
            let table = Table::new(rows).with(Style::rounded()).to_string();
            println!("{table}");
        }
        OutputFormat::Json => {
            let data = result.spreads.as_ref().map(|spreads| {
                spreads
                    .iter()
                    .map(|(id, spread)| (id.to_string(), json!(spread.to_string())))
                    .collect::<serde_json::Map<String, serde_json::Value>>()
            });
            crate::output::print_json(&data)?;
        }
    }
    Ok(())
}
