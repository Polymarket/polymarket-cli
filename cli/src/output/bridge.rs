use polymarket_client_sdk::bridge::types::{
    DepositResponse, DepositTransactionStatus, StatusResponse, SupportedAssetsResponse,
};
use serde_json::json;
use tabled::settings::Style;
use tabled::{Table, Tabled};

use super::{OutputFormat, detail_field, format_decimal, print_detail_table};

// --- Deposit ---

pub fn print_deposit(response: &DepositResponse, output: &OutputFormat) {
    match output {
        OutputFormat::Table => {
            let mut rows = Vec::new();
            detail_field!(rows, "EVM", format!("{}", response.address.evm));
            detail_field!(rows, "Solana", response.address.svm.clone());
            detail_field!(rows, "Bitcoin", response.address.btc.clone());
            if let Some(note) = &response.note {
                detail_field!(rows, "Note", note.clone());
            }
            print_detail_table(rows);
        }
        OutputFormat::Json => {
            let data = json!({
                "evm": format!("{}", response.address.evm),
                "svm": response.address.svm,
                "btc": response.address.btc,
                "note": response.note,
            });
            println!("{}", serde_json::to_string_pretty(&data).unwrap());
        }
    }
}

// --- Supported Assets ---

pub fn print_supported_assets(response: &SupportedAssetsResponse, output: &OutputFormat) {
    match output {
        OutputFormat::Table => {
            if response.supported_assets.is_empty() {
                println!("No supported assets found.");
                return;
            }
            #[derive(Tabled)]
            struct Row {
                #[tabled(rename = "Chain")]
                chain: String,
                #[tabled(rename = "Chain ID")]
                chain_id: String,
                #[tabled(rename = "Token")]
                token: String,
                #[tabled(rename = "Symbol")]
                symbol: String,
                #[tabled(rename = "Decimals")]
                decimals: String,
                #[tabled(rename = "Min Deposit")]
                min_deposit: String,
            }
            let rows: Vec<Row> = response
                .supported_assets
                .iter()
                .map(|a| Row {
                    chain: a.chain_name.clone(),
                    chain_id: a.chain_id.to_string(),
                    token: a.token.name.clone(),
                    symbol: a.token.symbol.clone(),
                    decimals: a.token.decimals.to_string(),
                    min_deposit: format_decimal(a.min_checkout_usd),
                })
                .collect();
            let table = Table::new(rows).with(Style::rounded()).to_string();
            println!("{table}");
        }
        OutputFormat::Json => {
            let data: Vec<_> = response
                .supported_assets
                .iter()
                .map(|a| {
                    json!({
                        "chain_id": a.chain_id,
                        "chain_name": a.chain_name,
                        "token_name": a.token.name,
                        "token_symbol": a.token.symbol,
                        "token_address": a.token.address,
                        "token_decimals": a.token.decimals,
                        "min_checkout_usd": a.min_checkout_usd.to_string(),
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&data).unwrap());
        }
    }
}

// --- Status ---

fn format_status(s: &DepositTransactionStatus) -> &'static str {
    match s {
        DepositTransactionStatus::DepositDetected => "Detected",
        DepositTransactionStatus::Processing => "Processing",
        DepositTransactionStatus::OriginTxConfirmed => "Confirmed",
        DepositTransactionStatus::Submitted => "Submitted",
        DepositTransactionStatus::Completed => "Completed",
        DepositTransactionStatus::Failed => "Failed",
        _ => "Unknown",
    }
}

pub fn print_status(response: &StatusResponse, output: &OutputFormat) {
    match output {
        OutputFormat::Table => {
            if response.transactions.is_empty() {
                println!("No transactions found.");
                return;
            }
            #[derive(Tabled)]
            struct Row {
                #[tabled(rename = "From Chain")]
                from_chain: String,
                #[tabled(rename = "To Chain")]
                to_chain: String,
                #[tabled(rename = "Token")]
                token: String,
                #[tabled(rename = "Amount")]
                amount: String,
                #[tabled(rename = "Status")]
                status: String,
                #[tabled(rename = "Tx Hash")]
                tx_hash: String,
            }
            let rows: Vec<Row> = response
                .transactions
                .iter()
                .map(|tx| Row {
                    from_chain: tx.from_chain_id.to_string(),
                    to_chain: tx.to_chain_id.to_string(),
                    token: super::truncate(&tx.from_token_address, 14),
                    amount: tx.from_amount_base_unit.to_string(),
                    status: format_status(&tx.status).into(),
                    tx_hash: tx
                        .tx_hash
                        .as_deref()
                        .map_or_else(|| "—".into(), |h| super::truncate(h, 14)),
                })
                .collect();
            let table = Table::new(rows).with(Style::rounded()).to_string();
            println!("{table}");
        }
        OutputFormat::Json => {
            let data: Vec<_> = response
                .transactions
                .iter()
                .map(|tx| {
                    json!({
                        "from_chain_id": tx.from_chain_id,
                        "from_token_address": tx.from_token_address,
                        "from_amount_base_unit": tx.from_amount_base_unit.to_string(),
                        "to_chain_id": tx.to_chain_id,
                        "to_token_address": format!("{}", tx.to_token_address),
                        "status": format_status(&tx.status),
                        "tx_hash": tx.tx_hash,
                        "created_time_ms": tx.created_time_ms,
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&data).unwrap());
        }
    }
}
