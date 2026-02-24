use polymarket_client_sdk::gamma::types::response::Market;
use polymarket_client_sdk::types::Decimal;
use tabled::settings::Style;
use tabled::{Table, Tabled};

use super::{detail_field, format_decimal, print_detail_table, truncate};

#[derive(Tabled)]
struct MarketRow {
    #[tabled(rename = "Question")]
    question: String,
    #[tabled(rename = "Price")]
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
    let price_yes =
        primary_outcome_price(m).map_or_else(
            || "—".into(),
            |(outcome, price)| match outcome {
                Some(outcome) => format!("{outcome}: {:.2}¢", price * Decimal::from(100)),
                None => format!("{:.2}¢", price * Decimal::from(100)),
            },
        );

    MarketRow {
        question: truncate(question, 60),
        price_yes,
        volume: m.volume_num.map_or_else(|| "—".into(), format_decimal),
        liquidity: m.liquidity_num.map_or_else(|| "—".into(), format_decimal),
        status: market_status(m).into(),
    }
}

fn primary_outcome_price(m: &Market) -> Option<(Option<String>, Decimal)> {
    let prices = m.outcome_prices.as_ref()?;

    if let Some(outcomes) = m.outcomes.as_ref() {
        outcomes
            .iter()
            .zip(prices.iter())
            .find(|(outcome, _)| outcome.eq_ignore_ascii_case("yes"))
            .or_else(|| outcomes.iter().zip(prices.iter()).next())
            .map(|(outcome, price)| (Some(outcome.clone()), *price))
    } else {
        prices.first().map(|price| (None, *price))
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_market(val: serde_json::Value) -> Market {
        serde_json::from_value(val).unwrap()
    }

    #[test]
    fn status_closed_overrides_active() {
        let m = make_market(json!({"id": "1", "closed": true, "active": true}));
        assert_eq!(market_status(&m), "Closed");
    }

    #[test]
    fn status_active_when_not_closed() {
        let m = make_market(json!({"id": "1", "closed": false, "active": true}));
        assert_eq!(market_status(&m), "Active");
    }

    #[test]
    fn status_inactive_when_fields_missing() {
        let m = make_market(json!({"id": "1"}));
        assert_eq!(market_status(&m), "Inactive");
    }

    #[test]
    fn status_inactive_when_both_false() {
        let m = make_market(json!({"id": "1", "closed": false, "active": false}));
        assert_eq!(market_status(&m), "Inactive");
    }

    #[test]
    fn row_missing_optionals_shows_dashes() {
        let row = market_to_row(&make_market(json!({"id": "1"})));
        assert_eq!(row.question, "—");
        assert_eq!(row.price_yes, "—");
        assert_eq!(row.volume, "—");
        assert_eq!(row.liquidity, "—");
        assert_eq!(row.status, "Inactive");
    }

    #[test]
    fn row_formats_price_as_cents() {
        let m = make_market(json!({
            "id": "1",
            "outcomes": "[\"Yes\",\"No\"]",
            "outcomePrices": "[\"0.65\",\"0.35\"]"
        }));
        assert_eq!(market_to_row(&m).price_yes, "Yes: 65.00¢");
    }

    #[test]
    fn row_prefers_yes_outcome_when_not_first() {
        let m = make_market(json!({
            "id": "1",
            "outcomes": "[\"No\",\"Yes\"]",
            "outcomePrices": "[\"0.35\",\"0.65\"]"
        }));
        assert_eq!(market_to_row(&m).price_yes, "Yes: 65.00¢");
    }

    #[test]
    fn row_uses_first_outcome_for_non_binary_market() {
        let m = make_market(json!({
            "id": "1",
            "outcomes": "[\"Long\",\"Short\"]",
            "outcomePrices": "[\"0.58\",\"0.42\"]"
        }));
        assert_eq!(market_to_row(&m).price_yes, "Long: 58.00¢");
    }

    #[test]
    fn row_shows_unlabeled_price_when_outcomes_missing() {
        let m = make_market(json!({
            "id": "1",
            "outcomePrices": "[\"0.65\",\"0.35\"]"
        }));
        assert_eq!(market_to_row(&m).price_yes, "65.00¢");
    }

    #[test]
    fn row_truncates_long_question() {
        let long_q = "a".repeat(100);
        let m = make_market(json!({"id": "1", "question": long_q}));
        let row = market_to_row(&m);
        assert_eq!(row.question.chars().count(), 60);
    }

    #[test]
    fn row_formats_volume_and_liquidity() {
        let m = make_market(json!({"id": "1", "volumeNum": "1500000", "liquidityNum": "2500"}));
        let row = market_to_row(&m);
        assert_eq!(row.volume, "$1.5M");
        assert_eq!(row.liquidity, "$2.5K");
    }

    #[test]
    fn row_propagates_status() {
        let m = make_market(json!({"id": "1", "active": true}));
        assert_eq!(market_to_row(&m).status, "Active");
    }
}
