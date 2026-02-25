#![allow(clippy::items_after_statements)]

mod account;
mod books;
mod markets;
mod orders;
mod prices;

/// Base64-encoded empty cursor returned by the CLOB API when there are no more pages.
const END_CURSOR: &str = "LTE=";

// Shared utility used by multiple submodules.
pub(crate) use super::OutputFormat;

pub use account::{
    print_account_status, print_api_keys, print_balance, print_create_api_key,
    print_current_rewards, print_delete_api_key, print_earnings, print_geoblock,
    print_market_reward, print_notifications, print_reward_percentages, print_rewards,
    print_server_time, print_user_earnings_markets,
};
pub use books::{print_last_trade, print_last_trades_prices, print_order_book, print_order_books};
pub use markets::{
    print_clob_market, print_clob_markets, print_fee_rate, print_neg_risk, print_price_history,
    print_simplified_markets, print_tick_size,
};
pub use orders::{
    print_cancel_result, print_order_detail, print_order_scoring, print_orders,
    print_orders_scoring, print_post_order_result, print_post_orders_result, print_trades,
};
pub use prices::{
    print_batch_prices, print_midpoint, print_midpoints, print_price, print_spread, print_spreads,
};

use serde_json::json;

pub fn print_ok(result: &str, output: &OutputFormat) -> anyhow::Result<()> {
    match output {
        OutputFormat::Table => println!("CLOB API: {result}"),
        OutputFormat::Json => {
            super::print_json(&json!({"status": result}))?;
        }
    }
    Ok(())
}
