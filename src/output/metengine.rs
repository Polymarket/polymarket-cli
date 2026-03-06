use serde_json::Value;
use tabled::settings::Style;
use tabled::{Table, Tabled};

use super::{OutputFormat, truncate};

fn str_field(v: &Value, key: &str) -> String {
    v.get(key)
        .and_then(Value::as_str)
        .unwrap_or("—")
        .to_string()
}

fn num_field(v: &Value, key: &str) -> String {
    v.get(key)
        .and_then(Value::as_f64)
        .map(|n| format!("{n:.2}"))
        .unwrap_or_else(|| "—".into())
}

fn int_field(v: &Value, key: &str) -> String {
    v.get(key)
        .and_then(Value::as_i64)
        .map(|n| n.to_string())
        .unwrap_or_else(|| "—".into())
}

fn usd_field(v: &Value, key: &str) -> String {
    v.get(key)
        .and_then(Value::as_f64)
        .map(|n| {
            if n >= 1_000_000.0 {
                format!("${:.1}M", n / 1_000_000.0)
            } else if n >= 1_000.0 {
                format!("${:.1}K", n / 1_000.0)
            } else {
                format!("${n:.2}")
            }
        })
        .unwrap_or_else(|| "—".into())
}

fn pct_field(v: &Value, key: &str) -> String {
    v.get(key)
        .and_then(Value::as_f64)
        .map(|n| format!("{n:.1}%"))
        .unwrap_or_else(|| "—".into())
}

fn print_json_raw(data: &Value) -> anyhow::Result<()> {
    super::print_json(data)
}

fn as_array(data: &Value) -> &[Value] {
    data.as_array().map(|v| v.as_slice()).unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Free endpoints
// ---------------------------------------------------------------------------

pub fn print_health(data: &Value, output: &OutputFormat) -> anyhow::Result<()> {
    match output {
        OutputFormat::Json => print_json_raw(data),
        OutputFormat::Table => {
            println!("MetEngine API: {}", str_field(data, "status"));
            Ok(())
        }
    }
}

pub fn print_pricing(data: &Value, output: &OutputFormat) -> anyhow::Result<()> {
    match output {
        OutputFormat::Json => print_json_raw(data),
        OutputFormat::Table => {
            let tiers = data
                .as_array()
                .or_else(|| data.get("tiers").and_then(|v| v.as_array()));
            if let Some(tiers) = tiers {
                #[derive(Tabled)]
                struct Row {
                    #[tabled(rename = "Tier")]
                    tier: String,
                    #[tabled(rename = "Price")]
                    price: String,
                    #[tabled(rename = "Endpoints")]
                    endpoints: String,
                }
                let rows: Vec<Row> = tiers
                    .iter()
                    .map(|t| Row {
                        tier: str_field(t, "name"),
                        price: str_field(t, "price"),
                        endpoints: int_field(t, "endpoint_count"),
                    })
                    .collect();
                println!("{}", Table::new(rows).with(Style::rounded()));
            } else {
                println!("{}", serde_json::to_string_pretty(data)?);
            }
            Ok(())
        }
    }
}

// ---------------------------------------------------------------------------
// Market Discovery
// ---------------------------------------------------------------------------

pub fn print_trending(data: &Value, output: &OutputFormat) -> anyhow::Result<()> {
    match output {
        OutputFormat::Json => print_json_raw(data),
        OutputFormat::Table => {
            let items = as_array(data);
            if items.is_empty() {
                println!("No trending markets found.");
                return Ok(());
            }
            #[derive(Tabled)]
            struct Row {
                #[tabled(rename = "Question")]
                question: String,
                #[tabled(rename = "Category")]
                category: String,
                #[tabled(rename = "Volume")]
                volume: String,
                #[tabled(rename = "Trades")]
                trades: String,
                #[tabled(rename = "Spike")]
                spike: String,
                #[tabled(rename = "Smart $")]
                smart_dir: String,
            }
            let rows: Vec<Row> = items
                .iter()
                .map(|m| Row {
                    question: truncate(&str_field(m, "question"), 45),
                    category: str_field(m, "category"),
                    volume: usd_field(m, "period_volume_usdc"),
                    trades: int_field(m, "period_trade_count"),
                    spike: num_field(m, "volume_spike_multiplier"),
                    smart_dir: str_field(m, "smart_money_net_direction"),
                })
                .collect();
            println!("{}", Table::new(rows).with(Style::rounded()));
            Ok(())
        }
    }
}

pub fn print_search(data: &Value, output: &OutputFormat) -> anyhow::Result<()> {
    match output {
        OutputFormat::Json => print_json_raw(data),
        OutputFormat::Table => {
            let items = as_array(data);
            if items.is_empty() {
                println!("No markets found.");
                return Ok(());
            }
            #[derive(Tabled)]
            struct Row {
                #[tabled(rename = "Question")]
                question: String,
                #[tabled(rename = "Category")]
                category: String,
                #[tabled(rename = "Volume")]
                volume: String,
                #[tabled(rename = "Smart $")]
                smart_outcome: String,
                #[tabled(rename = "Price")]
                price: String,
            }
            let rows: Vec<Row> = items
                .iter()
                .map(|m| Row {
                    question: truncate(&str_field(m, "question"), 50),
                    category: str_field(m, "category"),
                    volume: usd_field(m, "total_volume_usdc"),
                    smart_outcome: str_field(m, "smart_money_outcome"),
                    price: num_field(m, "leader_price"),
                })
                .collect();
            println!("{}", Table::new(rows).with(Style::rounded()));
            Ok(())
        }
    }
}

pub fn print_categories(data: &Value, output: &OutputFormat) -> anyhow::Result<()> {
    match output {
        OutputFormat::Json => print_json_raw(data),
        OutputFormat::Table => {
            let items = as_array(data);
            if items.is_empty() {
                println!("No categories found.");
                return Ok(());
            }
            #[derive(Tabled)]
            struct Row {
                #[tabled(rename = "Category")]
                name: String,
                #[tabled(rename = "Markets")]
                active: String,
                #[tabled(rename = "Volume")]
                volume: String,
                #[tabled(rename = "Trades")]
                trades: String,
                #[tabled(rename = "Traders")]
                traders: String,
            }
            let rows: Vec<Row> = items
                .iter()
                .map(|c| Row {
                    name: str_field(c, "name"),
                    active: int_field(c, "active_markets"),
                    volume: usd_field(c, "period_volume"),
                    trades: int_field(c, "period_trades"),
                    traders: int_field(c, "unique_traders"),
                })
                .collect();
            println!("{}", Table::new(rows).with(Style::rounded()));
            Ok(())
        }
    }
}

pub fn print_platform_stats(data: &Value, output: &OutputFormat) -> anyhow::Result<()> {
    match output {
        OutputFormat::Json => print_json_raw(data),
        OutputFormat::Table => {
            let rows = vec![
                ["Timeframe".into(), str_field(data, "timeframe")],
                ["Total Volume".into(), usd_field(data, "total_volume_usdc")],
                ["Total Trades".into(), int_field(data, "total_trades")],
                ["Active Traders".into(), int_field(data, "active_traders")],
                ["Active Markets".into(), int_field(data, "active_markets")],
                [
                    "Resolved Markets".into(),
                    int_field(data, "resolved_markets"),
                ],
                [
                    "Smart Wallets".into(),
                    int_field(data, "smart_wallet_count"),
                ],
                [
                    "Avg Trade Size".into(),
                    usd_field(data, "avg_trade_size_usdc"),
                ],
            ];
            super::print_detail_table(rows);
            Ok(())
        }
    }
}

pub fn print_price_history(data: &Value, output: &OutputFormat) -> anyhow::Result<()> {
    match output {
        OutputFormat::Json => print_json_raw(data),
        OutputFormat::Table => {
            println!("Market: {}", str_field(data, "question"));
            if let Some(candles_map) = data.get("candles_by_outcome").and_then(|v| v.as_object()) {
                for (outcome, candles) in candles_map {
                    println!("\nOutcome: {outcome}");
                    let items = candles.as_array().map(|v| v.as_slice()).unwrap_or_default();
                    #[derive(Tabled)]
                    struct Row {
                        #[tabled(rename = "Time")]
                        bucket: String,
                        #[tabled(rename = "Open")]
                        open: String,
                        #[tabled(rename = "High")]
                        high: String,
                        #[tabled(rename = "Low")]
                        low: String,
                        #[tabled(rename = "Close")]
                        close: String,
                        #[tabled(rename = "Volume")]
                        volume: String,
                    }
                    let rows: Vec<Row> = items
                        .iter()
                        .map(|c| Row {
                            bucket: str_field(c, "bucket"),
                            open: num_field(c, "open"),
                            high: num_field(c, "high"),
                            low: num_field(c, "low"),
                            close: num_field(c, "close"),
                            volume: usd_field(c, "volume"),
                        })
                        .collect();
                    println!("{}", Table::new(rows).with(Style::rounded()));
                }
            }
            Ok(())
        }
    }
}

pub fn print_similar(data: &Value, output: &OutputFormat) -> anyhow::Result<()> {
    match output {
        OutputFormat::Json => print_json_raw(data),
        OutputFormat::Table => {
            let items = as_array(data);
            if items.is_empty() {
                println!("No similar markets found.");
                return Ok(());
            }
            #[derive(Tabled)]
            struct Row {
                #[tabled(rename = "Question")]
                question: String,
                #[tabled(rename = "Overlap %")]
                overlap: String,
                #[tabled(rename = "Shared")]
                shared: String,
                #[tabled(rename = "Volume")]
                volume: String,
            }
            let rows: Vec<Row> = items
                .iter()
                .map(|m| Row {
                    question: truncate(&str_field(m, "question"), 45),
                    overlap: pct_field(m, "wallet_overlap_pct"),
                    shared: int_field(m, "shared_wallet_count"),
                    volume: usd_field(m, "total_volume_usdc"),
                })
                .collect();
            println!("{}", Table::new(rows).with(Style::rounded()));
            Ok(())
        }
    }
}

pub fn print_opportunities(data: &Value, output: &OutputFormat) -> anyhow::Result<()> {
    match output {
        OutputFormat::Json => print_json_raw(data),
        OutputFormat::Table => {
            let items = as_array(data);
            if items.is_empty() {
                println!("No opportunities found.");
                return Ok(());
            }
            #[derive(Tabled)]
            struct Row {
                #[tabled(rename = "Question")]
                question: String,
                #[tabled(rename = "Favors")]
                favors: String,
                #[tabled(rename = "Signal")]
                signal: String,
                #[tabled(rename = "Smart %")]
                smart_pct: String,
                #[tabled(rename = "Gap")]
                gap: String,
            }
            let rows: Vec<Row> = items
                .iter()
                .map(|m| Row {
                    question: truncate(&str_field(m, "question"), 40),
                    favors: str_field(m, "smart_money_favors"),
                    signal: str_field(m, "signal_strength"),
                    smart_pct: pct_field(m, "smart_money_percentage"),
                    gap: num_field(m, "price_signal_gap"),
                })
                .collect();
            println!("{}", Table::new(rows).with(Style::rounded()));
            Ok(())
        }
    }
}

pub fn print_high_conviction(data: &Value, output: &OutputFormat) -> anyhow::Result<()> {
    match output {
        OutputFormat::Json => print_json_raw(data),
        OutputFormat::Table => {
            let items = as_array(data);
            if items.is_empty() {
                println!("No high-conviction markets found.");
                return Ok(());
            }
            #[derive(Tabled)]
            struct Row {
                #[tabled(rename = "Question")]
                question: String,
                #[tabled(rename = "Favors")]
                favors: String,
                #[tabled(rename = "Score")]
                conviction: String,
                #[tabled(rename = "Smart $")]
                smart_usd: String,
                #[tabled(rename = "Wallets")]
                wallets: String,
            }
            let rows: Vec<Row> = items
                .iter()
                .map(|m| Row {
                    question: truncate(&str_field(m, "question"), 40),
                    favors: str_field(m, "favored_outcome"),
                    conviction: num_field(m, "conviction_score"),
                    smart_usd: usd_field(m, "total_smart_usdc"),
                    wallets: int_field(m, "smart_wallet_count"),
                })
                .collect();
            println!("{}", Table::new(rows).with(Style::rounded()));
            Ok(())
        }
    }
}

pub fn print_capital_flow(data: &Value, output: &OutputFormat) -> anyhow::Result<()> {
    match output {
        OutputFormat::Json => print_json_raw(data),
        OutputFormat::Table => {
            println!("Timeframe: {}", str_field(data, "timeframe"));
            println!("Total Net Flow: {}", usd_field(data, "total_net_flow"));
            println!(
                "Biggest Inflow: {} | Biggest Outflow: {}",
                str_field(data, "biggest_inflow"),
                str_field(data, "biggest_outflow")
            );
            let cats = data
                .get("categories")
                .and_then(|v| v.as_array())
                .map(|v| v.as_slice())
                .unwrap_or_default();
            if !cats.is_empty() {
                #[derive(Tabled)]
                struct Row {
                    #[tabled(rename = "Category")]
                    category: String,
                    #[tabled(rename = "Buy")]
                    buy: String,
                    #[tabled(rename = "Sell")]
                    sell: String,
                    #[tabled(rename = "Net Flow")]
                    net: String,
                    #[tabled(rename = "Trend")]
                    trend: String,
                }
                let rows: Vec<Row> = cats
                    .iter()
                    .map(|c| Row {
                        category: str_field(c, "category"),
                        buy: usd_field(c, "current_buy_volume"),
                        sell: usd_field(c, "current_sell_volume"),
                        net: usd_field(c, "current_net_flow"),
                        trend: str_field(c, "flow_trend"),
                    })
                    .collect();
                println!("{}", Table::new(rows).with(Style::rounded()));
            }
            Ok(())
        }
    }
}

pub fn print_volume_heatmap(data: &Value, output: &OutputFormat) -> anyhow::Result<()> {
    match output {
        OutputFormat::Json => print_json_raw(data),
        OutputFormat::Table => {
            println!("Total Volume: {}", usd_field(data, "total_volume"));
            println!("Total Trades: {}", int_field(data, "total_trades"));
            let items = data
                .get("breakdown")
                .and_then(|v| v.as_array())
                .map(|v| v.as_slice())
                .unwrap_or_default();
            if !items.is_empty() {
                #[derive(Tabled)]
                struct Row {
                    #[tabled(rename = "Label")]
                    label: String,
                    #[tabled(rename = "Volume")]
                    volume: String,
                    #[tabled(rename = "Trades")]
                    trades: String,
                    #[tabled(rename = "% Total")]
                    pct: String,
                    #[tabled(rename = "Trend")]
                    trend: String,
                }
                let rows: Vec<Row> = items
                    .iter()
                    .map(|b| Row {
                        label: str_field(b, "label"),
                        volume: usd_field(b, "volume"),
                        trades: int_field(b, "trade_count"),
                        pct: pct_field(b, "pct_of_total"),
                        trend: str_field(b, "trend"),
                    })
                    .collect();
                println!("{}", Table::new(rows).with(Style::rounded()));
            }
            Ok(())
        }
    }
}

pub fn print_resolutions(data: &Value, output: &OutputFormat) -> anyhow::Result<()> {
    match output {
        OutputFormat::Json => print_json_raw(data),
        OutputFormat::Table => {
            let items = as_array(data);
            if items.is_empty() {
                println!("No resolved markets found.");
                return Ok(());
            }
            #[derive(Tabled)]
            struct Row {
                #[tabled(rename = "Question")]
                question: String,
                #[tabled(rename = "Winner")]
                winner: String,
                #[tabled(rename = "Volume")]
                volume: String,
                #[tabled(rename = "SM Accuracy")]
                accuracy: String,
            }
            let rows: Vec<Row> = items
                .iter()
                .map(|m| Row {
                    question: truncate(&str_field(m, "question"), 45),
                    winner: str_field(m, "winning_outcome"),
                    volume: usd_field(m, "total_volume_usdc"),
                    accuracy: pct_field(m, "smart_money_accuracy"),
                })
                .collect();
            println!("{}", Table::new(rows).with(Style::rounded()));
            Ok(())
        }
    }
}

pub fn print_dumb_money(data: &Value, output: &OutputFormat) -> anyhow::Result<()> {
    match output {
        OutputFormat::Json => print_json_raw(data),
        OutputFormat::Table => {
            println!("Market: {}", str_field(data, "question"));
            if let Some(summary) = data.get("summary") {
                println!(
                    "Wallets: {} | Total USDC: {} | Avg Score: {} | Consensus: {}",
                    int_field(summary, "total_wallets"),
                    usd_field(summary, "total_usdc"),
                    num_field(summary, "avg_score"),
                    str_field(summary, "consensus_outcome"),
                );
            }
            let positions = data
                .get("positions")
                .and_then(|v| v.as_array())
                .map(|v| v.as_slice())
                .unwrap_or_default();
            if !positions.is_empty() {
                #[derive(Tabled)]
                struct Row {
                    #[tabled(rename = "Wallet")]
                    wallet: String,
                    #[tabled(rename = "Score")]
                    score: String,
                    #[tabled(rename = "Outcome")]
                    outcome: String,
                    #[tabled(rename = "USDC")]
                    usdc: String,
                }
                let rows: Vec<Row> = positions
                    .iter()
                    .map(|p| Row {
                        wallet: truncate(&str_field(p, "wallet"), 14),
                        score: num_field(p, "score"),
                        outcome: str_field(p, "outcome"),
                        usdc: usd_field(p, "usdc_invested"),
                    })
                    .collect();
                println!("{}", Table::new(rows).with(Style::rounded()));
            }
            Ok(())
        }
    }
}

// ---------------------------------------------------------------------------
// Intelligence / Trades
// ---------------------------------------------------------------------------

pub fn print_intelligence(data: &Value, output: &OutputFormat) -> anyhow::Result<()> {
    match output {
        OutputFormat::Json => print_json_raw(data),
        OutputFormat::Table => {
            println!("Market: {}", str_field(data, "question"));
            if let Some(sm) = data.get("smart_money") {
                println!(
                    "Smart Money Consensus: {} (strength: {})",
                    str_field(sm, "consensus_outcome"),
                    num_field(sm, "consensus_strength"),
                );
            }
            if let Some(sig) = data.get("signal_analysis") {
                println!("Signal: {}", str_field(sig, "signal_summary"));
            }
            if let Some(act) = data.get("recent_activity") {
                println!(
                    "24h: Volume {} | Trades {} | B/S Ratio {} | Trend: {}",
                    usd_field(act, "volume_24h"),
                    int_field(act, "trade_count_24h"),
                    num_field(act, "buy_sell_ratio"),
                    str_field(act, "volume_trend"),
                );
            }
            Ok(())
        }
    }
}

pub fn print_sentiment(data: &Value, output: &OutputFormat) -> anyhow::Result<()> {
    match output {
        OutputFormat::Json => print_json_raw(data),
        OutputFormat::Table => {
            println!("Market: {}", str_field(data, "question"));
            if let Some(overall) = data.get("overall_sentiment") {
                println!(
                    "Overall Sentiment: {} (score: {})",
                    str_field(overall, "label"),
                    num_field(overall, "score"),
                );
            }
            if let Some(momentum) = data.get("momentum") {
                println!(
                    "Momentum: {} (strength: {})",
                    str_field(momentum, "direction"),
                    num_field(momentum, "strength"),
                );
            }
            Ok(())
        }
    }
}

pub fn print_participants(data: &Value, output: &OutputFormat) -> anyhow::Result<()> {
    match output {
        OutputFormat::Json => print_json_raw(data),
        OutputFormat::Table => {
            println!("Market: {}", str_field(data, "question"));
            println!(
                "Total Wallets: {} | Total USDC: {}",
                int_field(data, "total_wallets"),
                usd_field(data, "total_usdc"),
            );
            if let Some(tiers) = data.get("tier_distribution").and_then(|v| v.as_object()) {
                println!("Tier Distribution:");
                for (tier, count) in tiers {
                    println!("  {tier}: {count}");
                }
            }
            Ok(())
        }
    }
}

pub fn print_insiders(data: &Value, output: &OutputFormat) -> anyhow::Result<()> {
    match output {
        OutputFormat::Json => print_json_raw(data),
        OutputFormat::Table => {
            println!("Market: {}", str_field(data, "question"));
            let items = data
                .get("insiders")
                .and_then(|v| v.as_array())
                .map(|v| v.as_slice())
                .unwrap_or_default();
            if items.is_empty() {
                println!("No insiders detected.");
                return Ok(());
            }
            #[derive(Tabled)]
            struct Row {
                #[tabled(rename = "Wallet")]
                wallet: String,
                #[tabled(rename = "Score")]
                score: String,
                #[tabled(rename = "Outcome")]
                outcome: String,
                #[tabled(rename = "USDC")]
                usdc: String,
                #[tabled(rename = "Age (d)")]
                age: String,
            }
            let rows: Vec<Row> = items
                .iter()
                .map(|i| Row {
                    wallet: truncate(&str_field(i, "wallet"), 14),
                    score: num_field(i, "insider_score"),
                    outcome: str_field(i, "outcome"),
                    usdc: usd_field(i, "buy_usdc"),
                    age: int_field(i, "wallet_age_days"),
                })
                .collect();
            println!("{}", Table::new(rows).with(Style::rounded()));
            Ok(())
        }
    }
}

pub fn print_trades(data: &Value, output: &OutputFormat) -> anyhow::Result<()> {
    match output {
        OutputFormat::Json => print_json_raw(data),
        OutputFormat::Table => {
            println!("Market: {}", str_field(data, "question"));
            println!(
                "Trades: {} | Volume: {}",
                int_field(data, "trade_count"),
                usd_field(data, "total_volume"),
            );
            let items = data
                .get("trades")
                .and_then(|v| v.as_array())
                .map(|v| v.as_slice())
                .unwrap_or_default();
            if !items.is_empty() {
                #[derive(Tabled)]
                struct Row {
                    #[tabled(rename = "Wallet")]
                    wallet: String,
                    #[tabled(rename = "Side")]
                    side: String,
                    #[tabled(rename = "Outcome")]
                    outcome: String,
                    #[tabled(rename = "USDC")]
                    usdc: String,
                    #[tabled(rename = "Price")]
                    price: String,
                    #[tabled(rename = "Score")]
                    score: String,
                }
                let rows: Vec<Row> = items
                    .iter()
                    .map(|t| Row {
                        wallet: truncate(&str_field(t, "wallet"), 14),
                        side: str_field(t, "side"),
                        outcome: str_field(t, "outcome"),
                        usdc: usd_field(t, "usdc_size"),
                        price: num_field(t, "price"),
                        score: num_field(t, "wallet_score"),
                    })
                    .collect();
                println!("{}", Table::new(rows).with(Style::rounded()));
            }
            Ok(())
        }
    }
}

pub fn print_whale_trades(data: &Value, output: &OutputFormat) -> anyhow::Result<()> {
    match output {
        OutputFormat::Json => print_json_raw(data),
        OutputFormat::Table => {
            let items = as_array(data);
            if items.is_empty() {
                println!("No whale trades found.");
                return Ok(());
            }
            #[derive(Tabled)]
            struct Row {
                #[tabled(rename = "Question")]
                question: String,
                #[tabled(rename = "Wallet")]
                wallet: String,
                #[tabled(rename = "Side")]
                side: String,
                #[tabled(rename = "Outcome")]
                outcome: String,
                #[tabled(rename = "USDC")]
                usdc: String,
                #[tabled(rename = "Score")]
                score: String,
            }
            let rows: Vec<Row> = items
                .iter()
                .map(|t| Row {
                    question: truncate(&str_field(t, "question"), 35),
                    wallet: truncate(&str_field(t, "wallet"), 14),
                    side: str_field(t, "side"),
                    outcome: str_field(t, "outcome"),
                    usdc: usd_field(t, "usdc_size"),
                    score: num_field(t, "wallet_score"),
                })
                .collect();
            println!("{}", Table::new(rows).with(Style::rounded()));
            Ok(())
        }
    }
}

pub fn print_alpha_callers(data: &Value, output: &OutputFormat) -> anyhow::Result<()> {
    match output {
        OutputFormat::Json => print_json_raw(data),
        OutputFormat::Table => {
            let items = as_array(data);
            if items.is_empty() {
                println!("No alpha callers found.");
                return Ok(());
            }
            #[derive(Tabled)]
            struct Row {
                #[tabled(rename = "Wallet")]
                wallet: String,
                #[tabled(rename = "Market")]
                market: String,
                #[tabled(rename = "Days Early")]
                days_early: String,
                #[tabled(rename = "Bet")]
                bet: String,
                #[tabled(rename = "Score")]
                score: String,
                #[tabled(rename = "Win Rate")]
                win_rate: String,
            }
            let rows: Vec<Row> = items
                .iter()
                .map(|a| Row {
                    wallet: truncate(&str_field(a, "wallet"), 14),
                    market: truncate(&str_field(a, "market_question"), 35),
                    days_early: int_field(a, "days_before_resolution"),
                    bet: usd_field(a, "bet_size_usdc"),
                    score: num_field(a, "wallet_score"),
                    win_rate: pct_field(a, "win_rate"),
                })
                .collect();
            println!("{}", Table::new(rows).with(Style::rounded()));
            Ok(())
        }
    }
}

pub fn print_wallet_insiders(data: &Value, output: &OutputFormat) -> anyhow::Result<()> {
    match output {
        OutputFormat::Json => print_json_raw(data),
        OutputFormat::Table => {
            let items = data
                .get("candidates")
                .and_then(|v| v.as_array())
                .map(|v| v.as_slice())
                .unwrap_or_default();
            if items.is_empty() {
                println!("No insider candidates found.");
                return Ok(());
            }
            println!("Total candidates: {}", int_field(data, "total_candidates"));
            #[derive(Tabled)]
            struct Row {
                #[tabled(rename = "Wallet")]
                wallet: String,
                #[tabled(rename = "Score")]
                score: String,
                #[tabled(rename = "Age (d)")]
                age: String,
                #[tabled(rename = "Markets")]
                markets: String,
                #[tabled(rename = "Buy USDC")]
                buy_usdc: String,
                #[tabled(rename = "Win Rate")]
                win_rate: String,
            }
            let rows: Vec<Row> = items
                .iter()
                .map(|c| Row {
                    wallet: truncate(&str_field(c, "wallet"), 14),
                    score: num_field(c, "insider_score"),
                    age: int_field(c, "wallet_age_days"),
                    markets: int_field(c, "markets_traded"),
                    buy_usdc: usd_field(c, "total_buy_usdc"),
                    win_rate: pct_field(c, "win_rate"),
                })
                .collect();
            println!("{}", Table::new(rows).with(Style::rounded()));
            Ok(())
        }
    }
}

// ---------------------------------------------------------------------------
// Wallet Analytics
// ---------------------------------------------------------------------------

pub fn print_wallet_profile(data: &Value, output: &OutputFormat) -> anyhow::Result<()> {
    match output {
        OutputFormat::Json => print_json_raw(data),
        OutputFormat::Table => {
            println!("Wallet: {}", str_field(data, "wallet"));
            if let Some(p) = data.get("profile") {
                let rows = vec![
                    ["Score".into(), num_field(p, "score")],
                    ["Tier".into(), str_field(p, "tier")],
                    ["Win Rate".into(), pct_field(p, "win_rate")],
                    ["Sharpe".into(), num_field(p, "sharpe")],
                    ["Total PnL".into(), usd_field(p, "total_pnl")],
                    ["Total Volume".into(), usd_field(p, "total_volume")],
                    ["Resolved".into(), int_field(p, "resolved_positions")],
                    ["Category".into(), str_field(p, "primary_category")],
                ];
                super::print_detail_table(rows);
            }
            Ok(())
        }
    }
}

pub fn print_wallet_activity(data: &Value, output: &OutputFormat) -> anyhow::Result<()> {
    match output {
        OutputFormat::Json => print_json_raw(data),
        OutputFormat::Table => {
            println!(
                "Wallet: {} (score: {})",
                str_field(data, "wallet"),
                num_field(data, "wallet_score"),
            );
            let items = data
                .get("trades")
                .and_then(|v| v.as_array())
                .map(|v| v.as_slice())
                .unwrap_or_default();
            if items.is_empty() {
                println!("No activity found.");
                return Ok(());
            }
            #[derive(Tabled)]
            struct Row {
                #[tabled(rename = "Market")]
                market: String,
                #[tabled(rename = "Side")]
                side: String,
                #[tabled(rename = "Outcome")]
                outcome: String,
                #[tabled(rename = "USDC")]
                usdc: String,
                #[tabled(rename = "Price")]
                price: String,
            }
            let rows: Vec<Row> = items
                .iter()
                .map(|t| Row {
                    market: truncate(&str_field(t, "question"), 40),
                    side: str_field(t, "side"),
                    outcome: str_field(t, "outcome"),
                    usdc: usd_field(t, "usdc_size"),
                    price: num_field(t, "price"),
                })
                .collect();
            println!("{}", Table::new(rows).with(Style::rounded()));
            Ok(())
        }
    }
}

pub fn print_wallet_pnl(data: &Value, output: &OutputFormat) -> anyhow::Result<()> {
    match output {
        OutputFormat::Json => print_json_raw(data),
        OutputFormat::Table => {
            println!("Wallet: {}", str_field(data, "wallet"));
            let rows = vec![
                ["Realized PnL".into(), usd_field(data, "total_realized_pnl")],
                ["Total Positions".into(), int_field(data, "total_positions")],
                ["Winning".into(), int_field(data, "winning_positions")],
                ["Losing".into(), int_field(data, "losing_positions")],
            ];
            super::print_detail_table(rows);
            Ok(())
        }
    }
}

pub fn print_wallet_compare(data: &Value, output: &OutputFormat) -> anyhow::Result<()> {
    match output {
        OutputFormat::Json => print_json_raw(data),
        OutputFormat::Table => {
            let wallets = data
                .get("wallets")
                .and_then(|v| v.as_array())
                .map(|v| v.as_slice())
                .unwrap_or_default();
            if wallets.is_empty() {
                println!("No wallet data.");
                return Ok(());
            }
            #[derive(Tabled)]
            struct Row {
                #[tabled(rename = "Wallet")]
                wallet: String,
                #[tabled(rename = "Score")]
                score: String,
                #[tabled(rename = "Win Rate")]
                win_rate: String,
                #[tabled(rename = "PnL")]
                pnl: String,
                #[tabled(rename = "Volume")]
                volume: String,
            }
            let rows: Vec<Row> = wallets
                .iter()
                .map(|w| {
                    let profile = w.get("profile").unwrap_or(w);
                    Row {
                        wallet: truncate(&str_field(w, "wallet"), 14),
                        score: num_field(profile, "score"),
                        win_rate: pct_field(profile, "win_rate"),
                        pnl: usd_field(profile, "total_pnl"),
                        volume: usd_field(profile, "total_volume"),
                    }
                })
                .collect();
            println!("{}", Table::new(rows).with(Style::rounded()));
            Ok(())
        }
    }
}

pub fn print_copy_traders(data: &Value, output: &OutputFormat) -> anyhow::Result<()> {
    match output {
        OutputFormat::Json => print_json_raw(data),
        OutputFormat::Table => {
            let items = as_array(data);
            if items.is_empty() {
                println!("No copy traders found.");
                return Ok(());
            }
            #[derive(Tabled)]
            struct Row {
                #[tabled(rename = "Wallet")]
                wallet: String,
                #[tabled(rename = "Overlap")]
                overlap: String,
                #[tabled(rename = "Avg Lag (s)")]
                lag: String,
                #[tabled(rename = "Score")]
                score: String,
            }
            let rows: Vec<Row> = items
                .iter()
                .map(|c| Row {
                    wallet: truncate(&str_field(c, "wallet"), 14),
                    overlap: int_field(c, "overlap_trades"),
                    lag: num_field(c, "avg_lag_seconds"),
                    score: num_field(c, "wallet_score"),
                })
                .collect();
            println!("{}", Table::new(rows).with(Style::rounded()));
            Ok(())
        }
    }
}

pub fn print_top_performers(data: &Value, output: &OutputFormat) -> anyhow::Result<()> {
    match output {
        OutputFormat::Json => print_json_raw(data),
        OutputFormat::Table => {
            let items = as_array(data);
            if items.is_empty() {
                println!("No top performers found.");
                return Ok(());
            }
            #[derive(Tabled)]
            struct Row {
                #[tabled(rename = "#")]
                rank: String,
                #[tabled(rename = "Wallet")]
                wallet: String,
                #[tabled(rename = "PnL")]
                pnl: String,
                #[tabled(rename = "ROI")]
                roi: String,
                #[tabled(rename = "Win Rate")]
                win_rate: String,
                #[tabled(rename = "Score")]
                score: String,
            }
            let rows: Vec<Row> = items
                .iter()
                .map(|w| Row {
                    rank: int_field(w, "rank"),
                    wallet: truncate(&str_field(w, "wallet"), 14),
                    pnl: usd_field(w, "period_pnl_usdc"),
                    roi: pct_field(w, "period_roi_percent"),
                    win_rate: pct_field(w, "period_win_rate"),
                    score: num_field(w, "overall_score"),
                })
                .collect();
            println!("{}", Table::new(rows).with(Style::rounded()));
            Ok(())
        }
    }
}

pub fn print_niche_experts(data: &Value, output: &OutputFormat) -> anyhow::Result<()> {
    match output {
        OutputFormat::Json => print_json_raw(data),
        OutputFormat::Table => {
            let items = as_array(data);
            if items.is_empty() {
                println!("No niche experts found.");
                return Ok(());
            }
            #[derive(Tabled)]
            struct Row {
                #[tabled(rename = "Wallet")]
                wallet: String,
                #[tabled(rename = "Sharpe")]
                sharpe: String,
                #[tabled(rename = "Win Rate")]
                win_rate: String,
                #[tabled(rename = "PnL")]
                pnl: String,
                #[tabled(rename = "Volume")]
                volume: String,
                #[tabled(rename = "Score")]
                score: String,
            }
            let rows: Vec<Row> = items
                .iter()
                .map(|w| Row {
                    wallet: truncate(&str_field(w, "wallet"), 14),
                    sharpe: num_field(w, "category_sharpe"),
                    win_rate: pct_field(w, "category_win_rate"),
                    pnl: usd_field(w, "category_pnl"),
                    volume: usd_field(w, "category_volume"),
                    score: num_field(w, "overall_score"),
                })
                .collect();
            println!("{}", Table::new(rows).with(Style::rounded()));
            Ok(())
        }
    }
}
