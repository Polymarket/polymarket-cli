use polymarket_client_sdk::auth::Credentials;
use polymarket_client_sdk::clob::types::response::{
    ApiKeysResponse, BalanceAllowanceResponse, BanStatusResponse, CurrentRewardResponse,
    GeoblockResponse, MarketRewardResponse, NotificationResponse, Page, RewardsPercentagesResponse,
    TotalUserEarningResponse, UserEarningResponse, UserRewardsEarningResponse,
};
use polymarket_client_sdk::types::Decimal;
use serde_json::json;
use tabled::settings::Style;
use tabled::{Table, Tabled};

use super::END_CURSOR;
use crate::output::{OutputFormat, format_decimal, truncate};

pub fn print_server_time(timestamp: i64, output: &OutputFormat) -> anyhow::Result<()> {
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
            crate::output::print_json(&json!({"timestamp": timestamp}))?;
        }
    }
    Ok(())
}

pub fn print_geoblock(result: &GeoblockResponse, output: &OutputFormat) -> anyhow::Result<()> {
    match output {
        OutputFormat::Table => {
            println!("Blocked: {}", result.blocked);
            println!("IP: {}", result.ip);
            println!("Country: {}", result.country);
            println!("Region: {}", result.region);
        }
        OutputFormat::Json => {
            crate::output::print_json(&json!({
                "blocked": result.blocked,
                "ip": result.ip,
                "country": result.country,
                "region": result.region,
            }))?;
        }
    }
    Ok(())
}

/// USDC uses 6 decimal places on-chain.
const USDC_DECIMALS: u32 = 6;

pub fn print_balance(
    result: &BalanceAllowanceResponse,
    is_collateral: bool,
    output: &OutputFormat,
) -> anyhow::Result<()> {
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
            crate::output::print_json(&data)?;
        }
    }
    Ok(())
}

pub fn print_notifications(
    result: &[NotificationResponse],
    output: &OutputFormat,
) -> anyhow::Result<()> {
    match output {
        OutputFormat::Table => {
            if result.is_empty() {
                println!("No notifications.");
                return Ok(());
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
            crate::output::print_json(&data)?;
        }
    }
    Ok(())
}

pub fn print_rewards(
    result: &Page<UserEarningResponse>,
    output: &OutputFormat,
) -> anyhow::Result<()> {
    match output {
        OutputFormat::Table => {
            if result.data.is_empty() {
                println!("No reward earnings found.");
                return Ok(());
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
            if result.next_cursor != END_CURSOR {
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
            crate::output::print_json(&wrapper)?;
        }
    }
    Ok(())
}

pub fn print_earnings(
    result: &[TotalUserEarningResponse],
    output: &OutputFormat,
) -> anyhow::Result<()> {
    match output {
        OutputFormat::Table => {
            if result.is_empty() {
                println!("No earnings data found.");
                return Ok(());
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
            crate::output::print_json(&data)?;
        }
    }
    Ok(())
}

pub fn print_user_earnings_markets(
    result: &[UserRewardsEarningResponse],
    output: &OutputFormat,
) -> anyhow::Result<()> {
    match output {
        OutputFormat::Table => {
            if result.is_empty() {
                println!("No earnings data found.");
                return Ok(());
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
            crate::output::print_json(&data)?;
        }
    }
    Ok(())
}

pub fn print_reward_percentages(
    result: &RewardsPercentagesResponse,
    output: &OutputFormat,
) -> anyhow::Result<()> {
    match output {
        OutputFormat::Table => {
            if result.is_empty() {
                println!("No reward percentages found.");
                return Ok(());
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
            crate::output::print_json(&data)?;
        }
    }
    Ok(())
}

pub fn print_current_rewards(
    result: &Page<CurrentRewardResponse>,
    output: &OutputFormat,
) -> anyhow::Result<()> {
    match output {
        OutputFormat::Table => {
            if result.data.is_empty() {
                println!("No current rewards found.");
                return Ok(());
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
            if result.next_cursor != END_CURSOR {
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
            crate::output::print_json(&wrapper)?;
        }
    }
    Ok(())
}

pub fn print_market_reward(
    result: &Page<MarketRewardResponse>,
    output: &OutputFormat,
) -> anyhow::Result<()> {
    match output {
        OutputFormat::Table => {
            if result.data.is_empty() {
                println!("No market reward data found.");
                return Ok(());
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
            if result.next_cursor != END_CURSOR {
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
            crate::output::print_json(&wrapper)?;
        }
    }
    Ok(())
}

pub fn print_api_keys(result: &ApiKeysResponse, output: &OutputFormat) -> anyhow::Result<()> {
    // SDK limitation: ApiKeysResponse.keys is private with no public accessor or Serialize impl.
    // We use Debug output as the only available representation.
    let debug = format!("{result:?}");
    match output {
        OutputFormat::Table => {
            println!("API Keys: {debug}");
        }
        OutputFormat::Json => {
            crate::output::print_json(&json!({"api_keys": debug}))?;
        }
    }
    Ok(())
}

pub fn print_delete_api_key(
    result: &serde_json::Value,
    output: &OutputFormat,
) -> anyhow::Result<()> {
    match output {
        OutputFormat::Table => println!("API key deleted: {result}"),
        OutputFormat::Json => {
            crate::output::print_json(result)?;
        }
    }
    Ok(())
}

pub fn print_create_api_key(result: &Credentials, output: &OutputFormat) -> anyhow::Result<()> {
    match output {
        OutputFormat::Table => {
            println!("API Key: {}", result.key());
            println!("Secret: [redacted]");
            println!("Passphrase: [redacted]");
        }
        OutputFormat::Json => {
            crate::output::print_json(&json!({
                "api_key": result.key().to_string(),
                "secret": "[redacted]",
                "passphrase": "[redacted]",
            }))?;
        }
    }
    Ok(())
}

pub fn print_account_status(
    result: &BanStatusResponse,
    output: &OutputFormat,
) -> anyhow::Result<()> {
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
            crate::output::print_json(&json!({"closed_only": result.closed_only}))?;
        }
    }
    Ok(())
}
