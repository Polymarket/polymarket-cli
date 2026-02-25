use polymarket_client_sdk::clob::types::response::{
    FeeRateResponse, MarketResponse, NegRiskResponse, Page, PriceHistoryResponse,
    SimplifiedMarketResponse, TickSizeResponse,
};
use serde_json::json;
use tabled::settings::Style;
use tabled::{Table, Tabled};

use super::END_CURSOR;
use crate::output::{OutputFormat, truncate};

pub fn print_clob_market(result: &MarketResponse, output: &OutputFormat) -> anyhow::Result<()> {
    match output {
        OutputFormat::Table => {
            let mut rows = vec![
                ["Question".into(), result.question.clone()],
                ["Description".into(), truncate(&result.description, 80)],
                ["Slug".into(), result.market_slug.clone()],
                [
                    "Condition ID".into(),
                    result.condition_id.map_or("—".into(), |c| c.to_string()),
                ],
                ["Active".into(), result.active.to_string()],
                ["Closed".into(), result.closed.to_string()],
                [
                    "Accepting Orders".into(),
                    result.accepting_orders.to_string(),
                ],
                [
                    "Min Order Size".into(),
                    result.minimum_order_size.to_string(),
                ],
                ["Min Tick Size".into(), result.minimum_tick_size.to_string()],
                ["Neg Risk".into(), result.neg_risk.to_string()],
                [
                    "End Date".into(),
                    result.end_date_iso.map_or("—".into(), |d| d.to_rfc3339()),
                ],
            ];
            for token in &result.tokens {
                rows.push([
                    format!("Token ({})", token.outcome),
                    format!(
                        "ID: {} | Price: {} | Winner: {}",
                        token.token_id, token.price, token.winner
                    ),
                ]);
            }
            crate::output::print_detail_table(rows);
        }
        OutputFormat::Json => {
            crate::output::print_json(result)?;
        }
    }
    Ok(())
}

pub fn print_clob_markets(
    result: &Page<MarketResponse>,
    output: &OutputFormat,
) -> anyhow::Result<()> {
    match output {
        OutputFormat::Table => {
            if result.data.is_empty() {
                println!("No markets found.");
                return Ok(());
            }
            #[derive(Tabled)]
            struct Row {
                #[tabled(rename = "Question")]
                question: String,
                #[tabled(rename = "Active")]
                active: String,
                #[tabled(rename = "Tokens")]
                tokens: String,
                #[tabled(rename = "Min Tick")]
                min_tick: String,
            }
            let rows: Vec<Row> = result
                .data
                .iter()
                .map(|m| Row {
                    question: truncate(&m.question, 50),
                    active: if m.active { "Yes" } else { "No" }.into(),
                    tokens: m.tokens.len().to_string(),
                    min_tick: m.minimum_tick_size.to_string(),
                })
                .collect();
            let table = Table::new(rows).with(Style::rounded()).to_string();
            println!("{table}");
            if result.next_cursor != END_CURSOR {
                println!("Next cursor: {}", result.next_cursor);
            }
        }
        OutputFormat::Json => {
            crate::output::print_json(result)?;
        }
    }
    Ok(())
}

pub fn print_simplified_markets(
    result: &Page<SimplifiedMarketResponse>,
    output: &OutputFormat,
) -> anyhow::Result<()> {
    match output {
        OutputFormat::Table => {
            if result.data.is_empty() {
                println!("No markets found.");
                return Ok(());
            }
            #[derive(Tabled)]
            struct Row {
                #[tabled(rename = "Condition ID")]
                condition_id: String,
                #[tabled(rename = "Tokens")]
                tokens: String,
                #[tabled(rename = "Active")]
                active: String,
                #[tabled(rename = "Closed")]
                closed: String,
                #[tabled(rename = "Orders")]
                accepting_orders: String,
            }
            let rows: Vec<Row> = result
                .data
                .iter()
                .map(|m| Row {
                    condition_id: m
                        .condition_id
                        .map_or("—".into(), |c| truncate(&c.to_string(), 14)),
                    tokens: m.tokens.len().to_string(),
                    active: if m.active { "Yes" } else { "No" }.into(),
                    closed: if m.closed { "Yes" } else { "No" }.into(),
                    accepting_orders: if m.accepting_orders { "Yes" } else { "No" }.into(),
                })
                .collect();
            let table = Table::new(rows).with(Style::rounded()).to_string();
            println!("{table}");
            if result.next_cursor != END_CURSOR {
                println!("Next cursor: {}", result.next_cursor);
            }
        }
        OutputFormat::Json => {
            crate::output::print_json(result)?;
        }
    }
    Ok(())
}

pub fn print_tick_size(result: &TickSizeResponse, output: &OutputFormat) -> anyhow::Result<()> {
    match output {
        OutputFormat::Table => {
            println!("Tick size: {}", result.minimum_tick_size.as_decimal());
        }
        OutputFormat::Json => {
            crate::output::print_json(&json!({
                "minimum_tick_size": result.minimum_tick_size.as_decimal().to_string(),
            }))?;
        }
    }
    Ok(())
}

pub fn print_fee_rate(result: &FeeRateResponse, output: &OutputFormat) -> anyhow::Result<()> {
    match output {
        OutputFormat::Table => {
            println!("Fee rate: {} bps", result.base_fee);
        }
        OutputFormat::Json => {
            crate::output::print_json(&json!({
                "base_fee_bps": result.base_fee,
            }))?;
        }
    }
    Ok(())
}

pub fn print_neg_risk(result: &NegRiskResponse, output: &OutputFormat) -> anyhow::Result<()> {
    match output {
        OutputFormat::Table => println!("Neg risk: {}", result.neg_risk),
        OutputFormat::Json => {
            crate::output::print_json(&json!({"neg_risk": result.neg_risk}))?;
        }
    }
    Ok(())
}

pub fn print_price_history(
    result: &PriceHistoryResponse,
    output: &OutputFormat,
) -> anyhow::Result<()> {
    match output {
        OutputFormat::Table => {
            if result.history.is_empty() {
                println!("No price history found.");
                return Ok(());
            }
            #[derive(Tabled)]
            struct Row {
                #[tabled(rename = "Timestamp")]
                timestamp: String,
                #[tabled(rename = "Price")]
                price: String,
            }
            let rows: Vec<Row> = result
                .history
                .iter()
                .map(|p| Row {
                    timestamp: chrono::DateTime::from_timestamp(p.t, 0)
                        .map_or(p.t.to_string(), |dt| {
                            dt.format("%Y-%m-%d %H:%M").to_string()
                        }),
                    price: p.p.to_string(),
                })
                .collect();
            let table = Table::new(rows).with(Style::rounded()).to_string();
            println!("{table}");
        }
        OutputFormat::Json => {
            let data: Vec<_> = result
                .history
                .iter()
                .map(|p| json!({"timestamp": p.t, "price": p.p.to_string()}))
                .collect();
            crate::output::print_json(&data)?;
        }
    }
    Ok(())
}
