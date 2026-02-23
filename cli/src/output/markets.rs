use polymarket_client_sdk::gamma::types::response::Market;
use polymarket_client_sdk::types::Decimal;
use tabled::settings::Style;
use tabled::{Table, Tabled};

use super::{detail_field, format_decimal, print_detail_table, truncate};

#[derive(Tabled)]
struct MarketRow {
    #[tabled(rename = "Question")]
    question: String,
    #[tabled(rename = "Price (Yes)")]
    price_yes: String,
    #[tabled(rename = "Volume")]
    volume: String,
    #[tabled(rename = "Liquidity")]
    liquidity: String,
    #[tabled(rename = "Status")]
    status: String,
}

fn market_status(m: &Market) -> &'static str {
    if m.closed == Some(true) {
        "Closed"
    } else if m.active == Some(true) {
        "Active"
    } else {
        "Inactive"
    }
}

fn market_to_row(m: &Market) -> MarketRow {
    let question = m.question.as_deref().unwrap_or("—");
    let price_yes = m
        .outcome_prices
        .as_ref()
        .and_then(|p| p.first())
        .map_or_else(|| "—".into(), |p| format!("{:.2}¢", p * Decimal::from(100)));

    MarketRow {
        question: truncate(question, 60),
        price_yes,
        volume: m.volume_num.map_or_else(|| "—".into(), format_decimal),
        liquidity: m.liquidity_num.map_or_else(|| "—".into(), format_decimal),
        status: market_status(m).into(),
    }
}

pub fn print_markets_table(markets: &[Market]) {
    if markets.is_empty() {
        println!("No markets found.");
        return;
    }
    let rows: Vec<MarketRow> = markets.iter().map(market_to_row).collect();
    let table = Table::new(rows).with(Style::rounded()).to_string();
    println!("{table}");
}

pub fn print_market_detail(m: &Market) {
    let mut rows: Vec<[String; 2]> = Vec::new();

    detail_field!(rows, "ID", m.id.clone());
    detail_field!(rows, "Question", m.question.clone().unwrap_or_default());
    detail_field!(rows, "Slug", m.slug.clone().unwrap_or_default());
    detail_field!(
        rows,
        "Outcomes",
        m.outcomes
            .as_ref()
            .map(|o| o.join(", "))
            .unwrap_or_default()
    );
    detail_field!(
        rows,
        "Prices",
        m.outcome_prices
            .as_ref()
            .map(|p| p
                .iter()
                .map(|v| format!("{v:.4}"))
                .collect::<Vec<_>>()
                .join(", "))
            .unwrap_or_default()
    );
    detail_field!(
        rows,
        "Volume",
        m.volume_num.map(format_decimal).unwrap_or_default()
    );
    detail_field!(
        rows,
        "Liquidity",
        m.liquidity_num.map(format_decimal).unwrap_or_default()
    );
    detail_field!(
        rows,
        "Volume (24hr)",
        m.volume_24hr.map(format_decimal).unwrap_or_default()
    );
    detail_field!(
        rows,
        "Best Bid",
        m.best_bid.map(|v| format!("{v:.4}")).unwrap_or_default()
    );
    detail_field!(
        rows,
        "Best Ask",
        m.best_ask.map(|v| format!("{v:.4}")).unwrap_or_default()
    );
    detail_field!(
        rows,
        "Spread",
        m.spread.map(|v| format!("{v:.4}")).unwrap_or_default()
    );
    detail_field!(
        rows,
        "Last Trade",
        m.last_trade_price
            .map(|v| format!("{v:.4}"))
            .unwrap_or_default()
    );
    detail_field!(rows, "Status", market_status(m).into());
    detail_field!(
        rows,
        "Condition ID",
        m.condition_id.map(|c| format!("{c}")).unwrap_or_default()
    );
    detail_field!(
        rows,
        "CLOB Token IDs",
        m.clob_token_ids
            .as_ref()
            .map(|ids| ids
                .iter()
                .map(|id| format!("{id}"))
                .collect::<Vec<_>>()
                .join(", "))
            .unwrap_or_default()
    );
    detail_field!(
        rows,
        "Start Date",
        m.start_date.map(|d| d.to_string()).unwrap_or_default()
    );
    detail_field!(
        rows,
        "End Date",
        m.end_date.map(|d| d.to_string()).unwrap_or_default()
    );
    detail_field!(
        rows,
        "Description",
        m.description.clone().unwrap_or_default()
    );
    detail_field!(
        rows,
        "Resolution Source",
        m.resolution_source.clone().unwrap_or_default()
    );

    print_detail_table(rows);
}
