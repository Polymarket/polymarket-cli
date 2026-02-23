use polymarket_client_sdk::auth::Credentials;
use polymarket_client_sdk::clob::types::response::{
    ApiKeysResponse, BalanceAllowanceResponse, BanStatusResponse, CancelOrdersResponse,
    CurrentRewardResponse, FeeRateResponse, GeoblockResponse, LastTradePriceResponse,
    LastTradesPricesResponse, MarketResponse, MarketRewardResponse, MidpointResponse,
    MidpointsResponse, NegRiskResponse, NotificationResponse, OpenOrderResponse,
    OrderBookSummaryResponse, OrderScoringResponse, OrdersScoringResponse, Page, PostOrderResponse,
    PriceHistoryResponse, PriceResponse, PricesResponse, RewardsPercentagesResponse,
    SimplifiedMarketResponse, SpreadResponse, SpreadsResponse, TickSizeResponse,
    TotalUserEarningResponse, TradeResponse, UserEarningResponse, UserRewardsEarningResponse,
};
use polymarket_client_sdk::types::Decimal;
use serde_json::json;
use tabled::settings::Style;
use tabled::{Table, Tabled};

use super::{OutputFormat, format_decimal, truncate};

// --- Ok ---

pub fn print_ok(result: &str, output: &OutputFormat) {
    match output {
        OutputFormat::Table => println!("CLOB API: {result}"),
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({"status": result})).unwrap()
            );
        }
    }
}

// --- Price ---

pub fn print_price(result: &PriceResponse, output: &OutputFormat) {
    match output {
        OutputFormat::Table => println!("Price: {}", result.price),
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({"price": result.price.to_string()})).unwrap()
            );
        }
    }
}

// --- Batch Prices ---

pub fn print_batch_prices(result: &PricesResponse, output: &OutputFormat) {
    match output {
        OutputFormat::Table => {
            let Some(prices) = &result.prices else {
                println!("No prices available.");
                return;
            };
            if prices.is_empty() {
                println!("No prices available.");
                return;
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
            println!("{}", serde_json::to_string_pretty(&data).unwrap());
        }
    }
}

// --- Midpoint ---

pub fn print_midpoint(result: &MidpointResponse, output: &OutputFormat) {
    match output {
        OutputFormat::Table => println!("Midpoint: {}", result.mid),
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({"midpoint": result.mid.to_string()})).unwrap()
            );
        }
    }
}

// --- Midpoints ---

pub fn print_midpoints(result: &MidpointsResponse, output: &OutputFormat) {
    match output {
        OutputFormat::Table => {
            if result.midpoints.is_empty() {
                println!("No midpoints available.");
                return;
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
            println!("{}", serde_json::to_string_pretty(&data).unwrap());
        }
    }
}

// --- Spread ---

pub fn print_spread(result: &SpreadResponse, output: &OutputFormat) {
    match output {
        OutputFormat::Table => println!("Spread: {}", result.spread),
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({"spread": result.spread.to_string()}))
                    .unwrap()
            );
        }
    }
}

// --- Spreads ---

pub fn print_spreads(result: &SpreadsResponse, output: &OutputFormat) {
    match output {
        OutputFormat::Table => {
            let Some(spreads) = &result.spreads else {
                println!("No spreads available.");
                return;
            };
            if spreads.is_empty() {
                println!("No spreads available.");
                return;
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
            println!("{}", serde_json::to_string_pretty(&data).unwrap());
        }
    }
}

// --- Order Book ---

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

pub fn print_order_book(result: &OrderBookSummaryResponse, output: &OutputFormat) {
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
            println!(
                "{}",
                serde_json::to_string_pretty(&order_book_to_json(result)).unwrap()
            );
        }
    }
}

// --- Order Books (batch) ---

pub fn print_order_books(result: &[OrderBookSummaryResponse], output: &OutputFormat) {
    match output {
        OutputFormat::Table => {
            if result.is_empty() {
                println!("No order books found.");
                return;
            }
            for (i, book) in result.iter().enumerate() {
                if i > 0 {
                    println!();
                }
                print_order_book(book, output);
            }
        }
        OutputFormat::Json => {
            let data: Vec<_> = result.iter().map(order_book_to_json).collect();
            println!("{}", serde_json::to_string_pretty(&data).unwrap());
        }
    }
}

// --- Last Trade ---

pub fn print_last_trade(result: &LastTradePriceResponse, output: &OutputFormat) {
    match output {
        OutputFormat::Table => println!("Last Trade: {} ({})", result.price, result.side),
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "price": result.price.to_string(),
                    "side": result.side.to_string(),
                }))
                .unwrap()
            );
        }
    }
}

// --- Last Trades Prices (batch) ---

pub fn print_last_trades_prices(result: &[LastTradesPricesResponse], output: &OutputFormat) {
    match output {
        OutputFormat::Table => {
            if result.is_empty() {
                println!("No last trade prices found.");
                return;
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
            println!("{}", serde_json::to_string_pretty(&data).unwrap());
        }
    }
}

// --- CLOB Market ---

pub fn print_clob_market(result: &MarketResponse, output: &OutputFormat) {
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
            super::print_detail_table(rows);
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(result).unwrap());
        }
    }
}

// --- CLOB Markets ---

pub fn print_clob_markets(result: &Page<MarketResponse>, output: &OutputFormat) {
    match output {
        OutputFormat::Table => {
            if result.data.is_empty() {
                println!("No markets found.");
                return;
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
            if result.next_cursor != "LTE=" {
                println!("Next cursor: {}", result.next_cursor);
            }
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(result).unwrap());
        }
    }
}

// --- Simplified Markets ---

pub fn print_simplified_markets(result: &Page<SimplifiedMarketResponse>, output: &OutputFormat) {
    match output {
        OutputFormat::Table => {
            if result.data.is_empty() {
                println!("No markets found.");
                return;
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
            if result.next_cursor != "LTE=" {
                println!("Next cursor: {}", result.next_cursor);
            }
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(result).unwrap());
        }
    }
}

// --- Tick Size ---

pub fn print_tick_size(result: &TickSizeResponse, output: &OutputFormat) {
    match output {
        OutputFormat::Table => {
            println!("Tick size: {}", result.minimum_tick_size.as_decimal());
        }
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "minimum_tick_size": result.minimum_tick_size.as_decimal().to_string(),
                }))
                .unwrap()
            );
        }
    }
}

// --- Fee Rate ---

pub fn print_fee_rate(result: &FeeRateResponse, output: &OutputFormat) {
    match output {
        OutputFormat::Table => {
            println!("Fee rate: {} bps", result.base_fee);
        }
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "base_fee_bps": result.base_fee,
                }))
                .unwrap()
            );
        }
    }
}

// --- Neg Risk ---

pub fn print_neg_risk(result: &NegRiskResponse, output: &OutputFormat) {
    match output {
        OutputFormat::Table => println!("Neg risk: {}", result.neg_risk),
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({"neg_risk": result.neg_risk})).unwrap()
            );
        }
    }
}

// --- Price History ---

pub fn print_price_history(result: &PriceHistoryResponse, output: &OutputFormat) {
    match output {
        OutputFormat::Table => {
            if result.history.is_empty() {
                println!("No price history found.");
                return;
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
            println!("{}", serde_json::to_string_pretty(&data).unwrap());
        }
    }
}

// --- Server Time ---

pub fn print_server_time(timestamp: i64, output: &OutputFormat) {
    match output {
        OutputFormat::Table => {
            let dt = chrono::DateTime::from_timestamp(timestamp, 0);
            match dt {
                Some(dt) => {
                    println!(
                        "Server time: {} ({timestamp})",
                        dt.format("%Y-%m-%d %H:%M:%S UTC")
                    );
                }
                None => println!("Server time: {timestamp}"),
            }
        }
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({"timestamp": timestamp})).unwrap()
            );
        }
    }
}

// --- Geoblock ---

pub fn print_geoblock(result: &GeoblockResponse, output: &OutputFormat) {
    match output {
        OutputFormat::Table => {
            println!("Blocked: {}", result.blocked);
            println!("IP: {}", result.ip);
            println!("Country: {}", result.country);
            println!("Region: {}", result.region);
        }
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "blocked": result.blocked,
                    "ip": result.ip,
                    "country": result.country,
                    "region": result.region,
                }))
                .unwrap()
            );
        }
    }
}

// --- Orders ---

pub fn print_orders(result: &Page<OpenOrderResponse>, output: &OutputFormat) {
    match output {
        OutputFormat::Table => {
            if result.data.is_empty() {
                println!("No open orders.");
                return;
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
            if result.next_cursor != "LTE=" {
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
            println!("{}", serde_json::to_string_pretty(&wrapper).unwrap());
        }
    }
}

// --- Order Detail ---

pub fn print_order_detail(result: &OpenOrderResponse, output: &OutputFormat) {
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
            super::print_detail_table(rows);
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
            println!("{}", serde_json::to_string_pretty(&data).unwrap());
        }
    }
}

// --- Post Order Result ---

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

pub fn print_post_order_result(result: &PostOrderResponse, output: &OutputFormat) {
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
            println!(
                "{}",
                serde_json::to_string_pretty(&post_order_to_json(result)).unwrap()
            );
        }
    }
}

// --- Post Orders Result (batch) ---

pub fn print_post_orders_result(results: &[PostOrderResponse], output: &OutputFormat) {
    match output {
        OutputFormat::Table => {
            for (i, r) in results.iter().enumerate() {
                if i > 0 {
                    println!("---");
                }
                print_post_order_result(r, output);
            }
        }
        OutputFormat::Json => {
            let data: Vec<_> = results.iter().map(post_order_to_json).collect();
            println!("{}", serde_json::to_string_pretty(&data).unwrap());
        }
    }
}

// --- Cancel Result ---

pub fn print_cancel_result(result: &CancelOrdersResponse, output: &OutputFormat) {
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
            println!("{}", serde_json::to_string_pretty(&data).unwrap());
        }
    }
}

// --- Trades ---

pub fn print_trades(result: &Page<TradeResponse>, output: &OutputFormat) {
    match output {
        OutputFormat::Table => {
            if result.data.is_empty() {
                println!("No trades found.");
                return;
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
            if result.next_cursor != "LTE=" {
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
            println!("{}", serde_json::to_string_pretty(&wrapper).unwrap());
        }
    }
}

// --- Balance ---

/// USDC uses 6 decimal places on-chain.
const USDC_DECIMALS: u32 = 6;

pub fn print_balance(
    result: &BalanceAllowanceResponse,
    is_collateral: bool,
    output: &OutputFormat,
) {
    let divisor = Decimal::from(10u64.pow(USDC_DECIMALS));
    let human_balance = result.balance / divisor;
    match output {
        OutputFormat::Table => {
            if is_collateral {
                println!("Balance: {}", format_decimal(human_balance));
            } else {
                println!("Balance: {human_balance} shares");
            }
            if !result.allowances.is_empty() {
                println!("Allowances:");
                for (addr, allowance) in &result.allowances {
                    println!("  {}: {allowance}", truncate(&addr.to_string(), 14));
                }
            }
        }
        OutputFormat::Json => {
            let allowances: serde_json::Map<String, serde_json::Value> = result
                .allowances
                .iter()
                .map(|(addr, val)| (addr.to_string(), json!(val)))
                .collect();
            let data = json!({
                "balance": human_balance.to_string(),
                "allowances": allowances,
            });
            println!("{}", serde_json::to_string_pretty(&data).unwrap());
        }
    }
}

// --- Notifications ---

pub fn print_notifications(result: &[NotificationResponse], output: &OutputFormat) {
    match output {
        OutputFormat::Table => {
            if result.is_empty() {
                println!("No notifications.");
                return;
            }
            #[derive(Tabled)]
            struct Row {
                #[tabled(rename = "Type")]
                notif_type: String,
                #[tabled(rename = "Question")]
                question: String,
                #[tabled(rename = "Side")]
                side: String,
                #[tabled(rename = "Price")]
                price: String,
                #[tabled(rename = "Size")]
                size: String,
            }
            let rows: Vec<Row> = result
                .iter()
                .map(|n| Row {
                    notif_type: n.r#type.to_string(),
                    question: truncate(&n.payload.question, 40),
                    side: n.payload.side.to_string(),
                    price: n.payload.price.to_string(),
                    size: n.payload.matched_size.to_string(),
                })
                .collect();
            let table = Table::new(rows).with(Style::rounded()).to_string();
            println!("{table}");
        }
        OutputFormat::Json => {
            let data: Vec<_> = result
                .iter()
                .map(|n| {
                    json!({
                        "type": n.r#type,
                        "question": n.payload.question,
                        "side": n.payload.side.to_string(),
                        "price": n.payload.price.to_string(),
                        "outcome": n.payload.outcome,
                        "matched_size": n.payload.matched_size.to_string(),
                        "original_size": n.payload.original_size.to_string(),
                        "order_id": n.payload.order_id,
                        "trade_id": n.payload.trade_id,
                        "market": n.payload.market.to_string(),
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&data).unwrap());
        }
    }
}

// --- Rewards ---

pub fn print_rewards(result: &Page<UserEarningResponse>, output: &OutputFormat) {
    match output {
        OutputFormat::Table => {
            if result.data.is_empty() {
                println!("No reward earnings found.");
                return;
            }
            #[derive(Tabled)]
            struct Row {
                #[tabled(rename = "Date")]
                date: String,
                #[tabled(rename = "Condition ID")]
                condition_id: String,
                #[tabled(rename = "Earnings")]
                earnings: String,
                #[tabled(rename = "Rate")]
                rate: String,
            }
            let rows: Vec<Row> = result
                .data
                .iter()
                .map(|e| Row {
                    date: e.date.to_string(),
                    condition_id: truncate(&e.condition_id.to_string(), 14),
                    earnings: format_decimal(e.earnings),
                    rate: e.asset_rate.to_string(),
                })
                .collect();
            let table = Table::new(rows).with(Style::rounded()).to_string();
            println!("{table}");
            if result.next_cursor != "LTE=" {
                println!("Next cursor: {}", result.next_cursor);
            }
        }
        OutputFormat::Json => {
            let data: Vec<_> = result
                .data
                .iter()
                .map(|e| {
                    json!({
                        "date": e.date.to_string(),
                        "condition_id": e.condition_id.to_string(),
                        "asset_address": e.asset_address.to_string(),
                        "maker_address": e.maker_address.to_string(),
                        "earnings": e.earnings.to_string(),
                        "asset_rate": e.asset_rate.to_string(),
                    })
                })
                .collect();
            let wrapper = json!({"data": data, "next_cursor": result.next_cursor});
            println!("{}", serde_json::to_string_pretty(&wrapper).unwrap());
        }
    }
}

// --- Earnings ---

pub fn print_earnings(result: &[TotalUserEarningResponse], output: &OutputFormat) {
    match output {
        OutputFormat::Table => {
            if result.is_empty() {
                println!("No earnings data found.");
                return;
            }
            for (i, e) in result.iter().enumerate() {
                if i > 0 {
                    println!("---");
                }
                println!("Date: {}", e.date);
                println!("Earnings: {}", format_decimal(e.earnings));
                println!("Asset Rate: {}", e.asset_rate);
                println!("Maker: {}", e.maker_address);
            }
        }
        OutputFormat::Json => {
            let data: Vec<_> = result
                .iter()
                .map(|e| {
                    json!({
                        "date": e.date.to_string(),
                        "asset_address": e.asset_address.to_string(),
                        "maker_address": e.maker_address.to_string(),
                        "earnings": e.earnings.to_string(),
                        "asset_rate": e.asset_rate.to_string(),
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&data).unwrap());
        }
    }
}

// --- User Earnings with Markets ---

pub fn print_user_earnings_markets(result: &[UserRewardsEarningResponse], output: &OutputFormat) {
    match output {
        OutputFormat::Table => {
            if result.is_empty() {
                println!("No earnings data found.");
                return;
            }
            #[derive(Tabled)]
            struct Row {
                #[tabled(rename = "Question")]
                question: String,
                #[tabled(rename = "Condition ID")]
                condition_id: String,
                #[tabled(rename = "Earn %")]
                earning_pct: String,
                #[tabled(rename = "Max Spread")]
                max_spread: String,
                #[tabled(rename = "Min Size")]
                min_size: String,
            }
            let rows: Vec<Row> = result
                .iter()
                .map(|e| Row {
                    question: truncate(&e.question, 40),
                    condition_id: truncate(&e.condition_id.to_string(), 14),
                    earning_pct: format!("{}%", e.earning_percentage),
                    max_spread: e.rewards_max_spread.to_string(),
                    min_size: e.rewards_min_size.to_string(),
                })
                .collect();
            let table = Table::new(rows).with(Style::rounded()).to_string();
            println!("{table}");
        }
        OutputFormat::Json => {
            let data: Vec<_> = result
                .iter()
                .map(|e| {
                    json!({
                        "condition_id": e.condition_id.to_string(),
                        "question": e.question,
                        "market_slug": e.market_slug,
                        "event_slug": e.event_slug,
                        "earning_percentage": e.earning_percentage.to_string(),
                        "rewards_max_spread": e.rewards_max_spread.to_string(),
                        "rewards_min_size": e.rewards_min_size.to_string(),
                        "market_competitiveness": e.market_competitiveness.to_string(),
                        "maker_address": e.maker_address.to_string(),
                        "tokens": e.tokens.iter().map(|t| json!({
                            "token_id": t.token_id.to_string(),
                            "outcome": t.outcome,
                            "price": t.price.to_string(),
                            "winner": t.winner,
                        })).collect::<Vec<_>>(),
                        "rewards_config": e.rewards_config.iter().map(|r| json!({
                            "asset_address": r.asset_address.to_string(),
                            "start_date": r.start_date.to_string(),
                            "end_date": r.end_date.to_string(),
                            "rate_per_day": r.rate_per_day.to_string(),
                            "total_rewards": r.total_rewards.to_string(),
                        })).collect::<Vec<_>>(),
                        "earnings": e.earnings.iter().map(|ear| json!({
                            "asset_address": ear.asset_address.to_string(),
                            "earnings": ear.earnings.to_string(),
                            "asset_rate": ear.asset_rate.to_string(),
                        })).collect::<Vec<_>>(),
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&data).unwrap());
        }
    }
}

// --- Reward Percentages ---

pub fn print_reward_percentages(result: &RewardsPercentagesResponse, output: &OutputFormat) {
    match output {
        OutputFormat::Table => {
            if result.is_empty() {
                println!("No reward percentages found.");
                return;
            }
            #[derive(Tabled)]
            struct Row {
                #[tabled(rename = "Market")]
                market: String,
                #[tabled(rename = "Percentage")]
                percentage: String,
            }
            let rows: Vec<Row> = result
                .iter()
                .map(|(market, pct)| Row {
                    market: truncate(market, 20),
                    percentage: format!("{pct}%"),
                })
                .collect();
            let table = Table::new(rows).with(Style::rounded()).to_string();
            println!("{table}");
        }
        OutputFormat::Json => {
            let data: serde_json::Map<String, serde_json::Value> = result
                .iter()
                .map(|(k, v)| (k.clone(), json!(v.to_string())))
                .collect();
            println!("{}", serde_json::to_string_pretty(&data).unwrap());
        }
    }
}

// --- Current Rewards ---

pub fn print_current_rewards(result: &Page<CurrentRewardResponse>, output: &OutputFormat) {
    match output {
        OutputFormat::Table => {
            if result.data.is_empty() {
                println!("No current rewards found.");
                return;
            }
            #[derive(Tabled)]
            struct Row {
                #[tabled(rename = "Condition ID")]
                condition_id: String,
                #[tabled(rename = "Max Spread")]
                max_spread: String,
                #[tabled(rename = "Min Size")]
                min_size: String,
                #[tabled(rename = "Configs")]
                configs: String,
            }
            let rows: Vec<Row> = result
                .data
                .iter()
                .map(|r| Row {
                    condition_id: truncate(&r.condition_id.to_string(), 14),
                    max_spread: r.rewards_max_spread.to_string(),
                    min_size: r.rewards_min_size.to_string(),
                    configs: r.rewards_config.len().to_string(),
                })
                .collect();
            let table = Table::new(rows).with(Style::rounded()).to_string();
            println!("{table}");
            if result.next_cursor != "LTE=" {
                println!("Next cursor: {}", result.next_cursor);
            }
        }
        OutputFormat::Json => {
            let data: Vec<_> = result
                .data
                .iter()
                .map(|r| {
                    json!({
                        "condition_id": r.condition_id.to_string(),
                        "rewards_max_spread": r.rewards_max_spread.to_string(),
                        "rewards_min_size": r.rewards_min_size.to_string(),
                        "rewards_config": r.rewards_config.iter().map(|c| json!({
                            "asset_address": c.asset_address.to_string(),
                            "start_date": c.start_date.to_string(),
                            "end_date": c.end_date.to_string(),
                            "rate_per_day": c.rate_per_day.to_string(),
                            "total_rewards": c.total_rewards.to_string(),
                        })).collect::<Vec<_>>(),
                    })
                })
                .collect();
            let wrapper = json!({"data": data, "next_cursor": result.next_cursor});
            println!("{}", serde_json::to_string_pretty(&wrapper).unwrap());
        }
    }
}

// --- Market Reward ---

pub fn print_market_reward(result: &Page<MarketRewardResponse>, output: &OutputFormat) {
    match output {
        OutputFormat::Table => {
            if result.data.is_empty() {
                println!("No market reward data found.");
                return;
            }
            for (i, r) in result.data.iter().enumerate() {
                if i > 0 {
                    println!("---");
                }
                println!("Question: {}", r.question);
                println!("Condition ID: {}", r.condition_id);
                println!("Slug: {}", r.market_slug);
                println!("Max Spread: {}", r.rewards_max_spread);
                println!("Min Size: {}", r.rewards_min_size);
                println!("Competitiveness: {}", r.market_competitiveness);
                for token in &r.tokens {
                    println!(
                        "  Token ({}): {} | Price: {}",
                        token.outcome, token.token_id, token.price
                    );
                }
            }
            if result.next_cursor != "LTE=" {
                println!("Next cursor: {}", result.next_cursor);
            }
        }
        OutputFormat::Json => {
            let data: Vec<_> = result
                .data
                .iter()
                .map(|r| {
                    json!({
                        "condition_id": r.condition_id.to_string(),
                        "question": r.question,
                        "market_slug": r.market_slug,
                        "event_slug": r.event_slug,
                        "rewards_max_spread": r.rewards_max_spread.to_string(),
                        "rewards_min_size": r.rewards_min_size.to_string(),
                        "market_competitiveness": r.market_competitiveness.to_string(),
                        "tokens": r.tokens.iter().map(|t| json!({
                            "token_id": t.token_id.to_string(),
                            "outcome": t.outcome,
                            "price": t.price.to_string(),
                            "winner": t.winner,
                        })).collect::<Vec<_>>(),
                        "rewards_config": r.rewards_config.iter().map(|c| json!({
                            "id": c.id,
                            "asset_address": c.asset_address.to_string(),
                            "start_date": c.start_date.to_string(),
                            "end_date": c.end_date.to_string(),
                            "rate_per_day": c.rate_per_day.to_string(),
                            "total_rewards": c.total_rewards.to_string(),
                            "total_days": c.total_days.to_string(),
                        })).collect::<Vec<_>>(),
                    })
                })
                .collect();
            let wrapper = json!({"data": data, "next_cursor": result.next_cursor});
            println!("{}", serde_json::to_string_pretty(&wrapper).unwrap());
        }
    }
}

// --- Order Scoring ---

pub fn print_order_scoring(result: &OrderScoringResponse, output: &OutputFormat) {
    match output {
        OutputFormat::Table => println!("Scoring: {}", result.scoring),
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({"scoring": result.scoring})).unwrap()
            );
        }
    }
}

// --- Orders Scoring ---

pub fn print_orders_scoring(result: &OrdersScoringResponse, output: &OutputFormat) {
    match output {
        OutputFormat::Table => {
            if result.is_empty() {
                println!("No scoring data.");
                return;
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
            println!("{}", serde_json::to_string_pretty(result).unwrap());
        }
    }
}

// --- API Keys ---

pub fn print_api_keys(result: &ApiKeysResponse, output: &OutputFormat) {
    // ApiKeysResponse.keys is private with no public accessor — Debug is the only option.
    // Strip the wrapper to show just the key list.
    let debug = format!("{result:?}");
    let keys_str = debug
        .strip_prefix("ApiKeysResponse { keys: ")
        .and_then(|s| s.strip_suffix(" }"))
        .unwrap_or(&debug);
    match output {
        OutputFormat::Table => {
            println!("API Keys: {keys_str}");
        }
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({"api_keys": keys_str})).unwrap()
            );
        }
    }
}

// --- Delete API Key ---

pub fn print_delete_api_key(result: &serde_json::Value, output: &OutputFormat) {
    match output {
        OutputFormat::Table => println!("API key deleted: {result}"),
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(result).unwrap());
        }
    }
}

// --- Create API Key ---

pub fn print_create_api_key(result: &Credentials, output: &OutputFormat) {
    match output {
        OutputFormat::Table => {
            println!("API Key: {}", result.key());
            println!("Secret: [redacted]");
            println!("Passphrase: [redacted]");
        }
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "api_key": result.key().to_string(),
                    "secret": "[redacted]",
                    "passphrase": "[redacted]",
                }))
                .unwrap()
            );
        }
    }
}

// --- Account Status ---

pub fn print_account_status(result: &BanStatusResponse, output: &OutputFormat) {
    match output {
        OutputFormat::Table => {
            println!(
                "Account status: {}",
                if result.closed_only {
                    "Closed-only mode (restricted)"
                } else {
                    "Active"
                }
            );
        }
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({"closed_only": result.closed_only})).unwrap()
            );
        }
    }
}
