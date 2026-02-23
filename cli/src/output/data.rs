use polymarket_client_sdk::data::types::response::{
    Activity, BuilderLeaderboardEntry, BuilderVolumeEntry, ClosedPosition, LiveVolume, Market,
    MetaHolder, OpenInterest, Position, Trade, Traded, TraderLeaderboardEntry, Value,
};
use serde_json::json;
use tabled::settings::Style;
use tabled::{Table, Tabled};

use super::{OutputFormat, format_decimal, truncate};

fn format_market(m: &Market) -> String {
    match m {
        Market::Global => "Global".into(),
        Market::Market(id) => id.to_string(),
        _ => "Unknown".into(),
    }
}

// --- Positions ---

pub fn print_positions(positions: &[Position], output: &OutputFormat) {
    match output {
        OutputFormat::Table => {
            if positions.is_empty() {
                println!("No positions found.");
                return;
            }
            #[derive(Tabled)]
            struct Row {
                #[tabled(rename = "Market")]
                title: String,
                #[tabled(rename = "Outcome")]
                outcome: String,
                #[tabled(rename = "Size")]
                size: String,
                #[tabled(rename = "Avg Price")]
                avg_price: String,
                #[tabled(rename = "Current")]
                current_value: String,
                #[tabled(rename = "PnL")]
                pnl: String,
            }
            let rows: Vec<Row> = positions
                .iter()
                .map(|p| Row {
                    title: truncate(&p.title, 40),
                    outcome: p.outcome.clone(),
                    size: format!("{:.2}", p.size),
                    avg_price: format!("{:.4}", p.avg_price),
                    current_value: format_decimal(p.current_value),
                    pnl: format!("{:.2}", p.cash_pnl),
                })
                .collect();
            let table = Table::new(rows).with(Style::rounded()).to_string();
            println!("{table}");
        }
        OutputFormat::Json => {
            let data: Vec<_> = positions
                .iter()
                .map(|p| {
                    json!({
                        "title": p.title,
                        "slug": p.slug,
                        "outcome": p.outcome,
                        "outcome_index": p.outcome_index,
                        "size": p.size.to_string(),
                        "avg_price": p.avg_price.to_string(),
                        "initial_value": p.initial_value.to_string(),
                        "current_value": p.current_value.to_string(),
                        "cash_pnl": p.cash_pnl.to_string(),
                        "percent_pnl": p.percent_pnl.to_string(),
                        "realized_pnl": p.realized_pnl.to_string(),
                        "cur_price": p.cur_price.to_string(),
                        "condition_id": p.condition_id.to_string(),
                        "proxy_wallet": p.proxy_wallet.to_string(),
                        "redeemable": p.redeemable,
                        "mergeable": p.mergeable,
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&data).unwrap());
        }
    }
}

// --- Closed Positions ---

pub fn print_closed_positions(positions: &[ClosedPosition], output: &OutputFormat) {
    match output {
        OutputFormat::Table => {
            if positions.is_empty() {
                println!("No closed positions found.");
                return;
            }
            #[derive(Tabled)]
            struct Row {
                #[tabled(rename = "Market")]
                title: String,
                #[tabled(rename = "Outcome")]
                outcome: String,
                #[tabled(rename = "Avg Price")]
                avg_price: String,
                #[tabled(rename = "Realized PnL")]
                realized_pnl: String,
            }
            let rows: Vec<Row> = positions
                .iter()
                .map(|p| Row {
                    title: truncate(&p.title, 40),
                    outcome: p.outcome.clone(),
                    avg_price: format!("{:.4}", p.avg_price),
                    realized_pnl: format!("{:.2}", p.realized_pnl),
                })
                .collect();
            let table = Table::new(rows).with(Style::rounded()).to_string();
            println!("{table}");
        }
        OutputFormat::Json => {
            let data: Vec<_> = positions
                .iter()
                .map(|p| {
                    json!({
                        "title": p.title,
                        "slug": p.slug,
                        "outcome": p.outcome,
                        "outcome_index": p.outcome_index,
                        "avg_price": p.avg_price.to_string(),
                        "total_bought": p.total_bought.to_string(),
                        "realized_pnl": p.realized_pnl.to_string(),
                        "cur_price": p.cur_price.to_string(),
                        "condition_id": p.condition_id.to_string(),
                        "proxy_wallet": p.proxy_wallet.to_string(),
                        "timestamp": p.timestamp,
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&data).unwrap());
        }
    }
}

// --- Value ---

pub fn print_value(values: &[Value], output: &OutputFormat) {
    match output {
        OutputFormat::Table => {
            if values.is_empty() {
                println!("No value data found.");
                return;
            }
            #[derive(Tabled)]
            struct Row {
                #[tabled(rename = "User")]
                user: String,
                #[tabled(rename = "Value")]
                value: String,
            }
            let rows: Vec<Row> = values
                .iter()
                .map(|v| Row {
                    user: truncate(&v.user.to_string(), 14),
                    value: format_decimal(v.value),
                })
                .collect();
            let table = Table::new(rows).with(Style::rounded()).to_string();
            println!("{table}");
        }
        OutputFormat::Json => {
            let data: Vec<_> = values
                .iter()
                .map(|v| json!({"user": v.user.to_string(), "value": v.value.to_string()}))
                .collect();
            println!("{}", serde_json::to_string_pretty(&data).unwrap());
        }
    }
}

// --- Traded ---

pub fn print_traded(t: &Traded, output: &OutputFormat) {
    match output {
        OutputFormat::Table => println!("{}: {} markets traded", t.user, t.traded),
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "user": t.user.to_string(),
                    "traded": t.traded,
                }))
                .unwrap()
            );
        }
    }
}

// --- Trades ---

pub fn print_trades(trades: &[Trade], output: &OutputFormat) {
    match output {
        OutputFormat::Table => {
            if trades.is_empty() {
                println!("No trades found.");
                return;
            }
            #[derive(Tabled)]
            struct Row {
                #[tabled(rename = "Market")]
                title: String,
                #[tabled(rename = "Side")]
                side: String,
                #[tabled(rename = "Outcome")]
                outcome: String,
                #[tabled(rename = "Size")]
                size: String,
                #[tabled(rename = "Price")]
                price: String,
            }
            let rows: Vec<Row> = trades
                .iter()
                .map(|t| Row {
                    title: truncate(&t.title, 40),
                    side: t.side.to_string(),
                    outcome: t.outcome.clone(),
                    size: format!("{:.2}", t.size),
                    price: format!("{:.4}", t.price),
                })
                .collect();
            let table = Table::new(rows).with(Style::rounded()).to_string();
            println!("{table}");
        }
        OutputFormat::Json => {
            let data: Vec<_> = trades
                .iter()
                .map(|t| {
                    json!({
                        "title": t.title,
                        "slug": t.slug,
                        "side": t.side.to_string(),
                        "outcome": t.outcome,
                        "outcome_index": t.outcome_index,
                        "size": t.size.to_string(),
                        "price": t.price.to_string(),
                        "timestamp": t.timestamp,
                        "condition_id": t.condition_id.to_string(),
                        "proxy_wallet": t.proxy_wallet.to_string(),
                        "transaction_hash": t.transaction_hash.to_string(),
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&data).unwrap());
        }
    }
}

// --- Activity ---

pub fn print_activity(activity: &[Activity], output: &OutputFormat) {
    match output {
        OutputFormat::Table => {
            if activity.is_empty() {
                println!("No activity found.");
                return;
            }
            #[derive(Tabled)]
            struct Row {
                #[tabled(rename = "Type")]
                activity_type: String,
                #[tabled(rename = "Market")]
                title: String,
                #[tabled(rename = "Size")]
                size: String,
                #[tabled(rename = "USDC")]
                usdc_size: String,
                #[tabled(rename = "Tx")]
                tx: String,
            }
            let rows: Vec<Row> = activity
                .iter()
                .map(|a| Row {
                    activity_type: a.activity_type.to_string(),
                    title: truncate(a.title.as_deref().unwrap_or("—"), 35),
                    size: format!("{:.2}", a.size),
                    usdc_size: format_decimal(a.usdc_size),
                    tx: truncate(&a.transaction_hash.to_string(), 14),
                })
                .collect();
            let table = Table::new(rows).with(Style::rounded()).to_string();
            println!("{table}");
        }
        OutputFormat::Json => {
            let data: Vec<_> = activity
                .iter()
                .map(|a| {
                    json!({
                        "activity_type": a.activity_type.to_string(),
                        "title": a.title,
                        "size": a.size.to_string(),
                        "usdc_size": a.usdc_size.to_string(),
                        "timestamp": a.timestamp,
                        "transaction_hash": a.transaction_hash.to_string(),
                        "proxy_wallet": a.proxy_wallet.to_string(),
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&data).unwrap());
        }
    }
}

// --- Holders ---

pub fn print_holders(meta_holders: &[MetaHolder], output: &OutputFormat) {
    match output {
        OutputFormat::Table => {
            if meta_holders.is_empty() {
                println!("No holders found.");
                return;
            }
            #[derive(Tabled)]
            struct Row {
                #[tabled(rename = "Wallet")]
                wallet: String,
                #[tabled(rename = "Name")]
                name: String,
                #[tabled(rename = "Amount")]
                amount: String,
                #[tabled(rename = "Outcome")]
                outcome_index: String,
            }
            let rows: Vec<Row> = meta_holders
                .iter()
                .flat_map(|mh| {
                    mh.holders.iter().map(|h| Row {
                        wallet: truncate(&h.proxy_wallet.to_string(), 14),
                        name: h
                            .name
                            .as_deref()
                            .or(h.pseudonym.as_deref())
                            .unwrap_or("—")
                            .into(),
                        amount: format_decimal(h.amount),
                        outcome_index: h.outcome_index.to_string(),
                    })
                })
                .collect();
            let table = Table::new(rows).with(Style::rounded()).to_string();
            println!("{table}");
        }
        OutputFormat::Json => {
            let data: Vec<_> = meta_holders
                .iter()
                .map(|mh| {
                    let holders: Vec<_> = mh
                        .holders
                        .iter()
                        .map(|h| {
                            json!({
                                "proxy_wallet": h.proxy_wallet.to_string(),
                                "name": h.name,
                                "pseudonym": h.pseudonym,
                                "amount": h.amount.to_string(),
                                "outcome_index": h.outcome_index,
                            })
                        })
                        .collect();
                    json!({"token": mh.token.to_string(), "holders": holders})
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&data).unwrap());
        }
    }
}

// --- Open Interest ---

pub fn print_open_interest(oi: &[OpenInterest], output: &OutputFormat) {
    match output {
        OutputFormat::Table => {
            if oi.is_empty() {
                println!("No open interest data found.");
                return;
            }
            #[derive(Tabled)]
            struct Row {
                #[tabled(rename = "Market")]
                market: String,
                #[tabled(rename = "Open Interest")]
                value: String,
            }
            let rows: Vec<Row> = oi
                .iter()
                .map(|o| Row {
                    market: format_market(&o.market),
                    value: format_decimal(o.value),
                })
                .collect();
            let table = Table::new(rows).with(Style::rounded()).to_string();
            println!("{table}");
        }
        OutputFormat::Json => {
            let data: Vec<_> = oi
                .iter()
                .map(|o| json!({"market": format_market(&o.market), "value": o.value.to_string()}))
                .collect();
            println!("{}", serde_json::to_string_pretty(&data).unwrap());
        }
    }
}

// --- Live Volume ---

pub fn print_live_volume(volume: &[LiveVolume], output: &OutputFormat) {
    match output {
        OutputFormat::Table => {
            if volume.is_empty() {
                println!("No volume data found.");
                return;
            }
            #[derive(Tabled)]
            struct Row {
                #[tabled(rename = "Market")]
                market: String,
                #[tabled(rename = "Volume")]
                value: String,
            }
            for v in volume {
                println!("Total: {}", format_decimal(v.total));
                let rows: Vec<Row> = v
                    .markets
                    .iter()
                    .map(|mv| Row {
                        market: format_market(&mv.market),
                        value: format_decimal(mv.value),
                    })
                    .collect();
                let table = Table::new(rows).with(Style::rounded()).to_string();
                println!("{table}");
            }
        }
        OutputFormat::Json => {
            let data: Vec<_> = volume
                .iter()
                .map(|v| {
                    let markets: Vec<_> = v
                        .markets
                        .iter()
                        .map(|mv| json!({"market": format_market(&mv.market), "value": mv.value.to_string()}))
                        .collect();
                    json!({"total": v.total.to_string(), "markets": markets})
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&data).unwrap());
        }
    }
}

// --- Trader Leaderboard ---

pub fn print_leaderboard(entries: &[TraderLeaderboardEntry], output: &OutputFormat) {
    match output {
        OutputFormat::Table => {
            if entries.is_empty() {
                println!("No leaderboard entries found.");
                return;
            }
            #[derive(Tabled)]
            struct Row {
                #[tabled(rename = "#")]
                rank: String,
                #[tabled(rename = "Trader")]
                trader: String,
                #[tabled(rename = "PnL")]
                pnl: String,
                #[tabled(rename = "Volume")]
                volume: String,
            }
            let rows: Vec<Row> = entries
                .iter()
                .map(|e| Row {
                    rank: e.rank.to_string(),
                    trader: truncate(e.user_name.as_deref().unwrap_or("—"), 20),
                    pnl: format_decimal(e.pnl),
                    volume: format_decimal(e.vol),
                })
                .collect();
            let table = Table::new(rows).with(Style::rounded()).to_string();
            println!("{table}");
        }
        OutputFormat::Json => {
            let data: Vec<_> = entries
                .iter()
                .map(|e| {
                    json!({
                        "rank": e.rank,
                        "proxy_wallet": e.proxy_wallet.to_string(),
                        "user_name": e.user_name,
                        "pnl": e.pnl.to_string(),
                        "volume": e.vol.to_string(),
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&data).unwrap());
        }
    }
}

// --- Builder Leaderboard ---

pub fn print_builder_leaderboard(entries: &[BuilderLeaderboardEntry], output: &OutputFormat) {
    match output {
        OutputFormat::Table => {
            if entries.is_empty() {
                println!("No builder leaderboard entries found.");
                return;
            }
            #[derive(Tabled)]
            struct Row {
                #[tabled(rename = "#")]
                rank: String,
                #[tabled(rename = "Builder")]
                builder: String,
                #[tabled(rename = "Volume")]
                volume: String,
                #[tabled(rename = "Users")]
                active_users: String,
            }
            let rows: Vec<Row> = entries
                .iter()
                .map(|e| Row {
                    rank: e.rank.to_string(),
                    builder: truncate(&e.builder, 25),
                    volume: format_decimal(e.volume),
                    active_users: e.active_users.to_string(),
                })
                .collect();
            let table = Table::new(rows).with(Style::rounded()).to_string();
            println!("{table}");
        }
        OutputFormat::Json => {
            let data: Vec<_> = entries
                .iter()
                .map(|e| {
                    json!({
                        "rank": e.rank,
                        "builder": e.builder,
                        "volume": e.volume.to_string(),
                        "active_users": e.active_users,
                        "verified": e.verified,
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&data).unwrap());
        }
    }
}

// --- Builder Volume ---

pub fn print_builder_volume(entries: &[BuilderVolumeEntry], output: &OutputFormat) {
    match output {
        OutputFormat::Table => {
            if entries.is_empty() {
                println!("No builder volume data found.");
                return;
            }
            #[derive(Tabled)]
            struct Row {
                #[tabled(rename = "Date")]
                date: String,
                #[tabled(rename = "Builder")]
                builder: String,
                #[tabled(rename = "Volume")]
                volume: String,
                #[tabled(rename = "Users")]
                active_users: String,
                #[tabled(rename = "#")]
                rank: String,
            }
            let rows: Vec<Row> = entries
                .iter()
                .map(|e| Row {
                    date: e.dt.format("%Y-%m-%d").to_string(),
                    builder: truncate(&e.builder, 25),
                    volume: format_decimal(e.volume),
                    active_users: e.active_users.to_string(),
                    rank: e.rank.to_string(),
                })
                .collect();
            let table = Table::new(rows).with(Style::rounded()).to_string();
            println!("{table}");
        }
        OutputFormat::Json => {
            let data: Vec<_> = entries
                .iter()
                .map(|e| {
                    json!({
                        "date": e.dt.to_rfc3339(),
                        "builder": e.builder,
                        "volume": e.volume.to_string(),
                        "active_users": e.active_users,
                        "rank": e.rank,
                        "verified": e.verified,
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&data).unwrap());
        }
    }
}
