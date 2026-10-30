use alloy::primitives::U256;
use anyhow::Result;
use tabled::Tabled;
use tabled::settings::Style;

use super::OutputFormat;

pub struct ApprovalStatus {
    pub contract_name: String,
    pub contract_address: String,
    pub collateral_allowance: U256,
    pub ctf_approved: Option<bool>,
    pub collateral_error: Option<String>,
    pub ctf_error: Option<String>,
}

#[derive(Tabled)]
struct ApprovalRow {
    #[tabled(rename = "Contract")]
    contract: String,
    #[tabled(rename = "pUSD")]
    collateral: String,
    #[tabled(rename = "CTF Tokens")]
    ctf: String,
}

fn format_allowance(allowance: U256) -> String {
    if allowance == U256::MAX {
        "\u{2713} Unlimited".to_string()
    } else if allowance == U256::ZERO {
        "\u{2717} None".to_string()
    } else {
        let collateral_decimals = U256::from(10u64.pow(crate::commands::COLLATERAL_DECIMALS));
        let whole = allowance / collateral_decimals;
        format!("\u{2713} {whole} {}", crate::commands::COLLATERAL_SYMBOL)
    }
}

fn format_ctf(approved: Option<bool>) -> String {
    match approved {
        Some(true) => "\u{2713} Approved".to_string(),
        Some(false) => "\u{2717} Not set".to_string(),
        None => "N/A".to_string(),
    }
}

pub fn print_approval_status(statuses: &[ApprovalStatus], output: &OutputFormat) -> Result<()> {
    match output {
        OutputFormat::Json => {
            let json: Vec<serde_json::Value> = statuses
                .iter()
                .map(|s| {
                    let mut obj = serde_json::json!({
                        "contract": s.contract_name,
                        "address": s.contract_address,
                        "collateral": crate::commands::COLLATERAL_SYMBOL,
                        "collateral_allowance": s.collateral_allowance.to_string(),
                        "collateral_approved": s.collateral_allowance > U256::ZERO,
                        "ctf_required": s.ctf_approved.is_some(),
                        "ctf_approved": s.ctf_approved,
                    });
                    if let Some(ref err) = s.collateral_error {
                        obj["collateral_error"] = serde_json::Value::String(err.clone());
                    }
                    if let Some(ref err) = s.ctf_error {
                        obj["ctf_error"] = serde_json::Value::String(err.clone());
                    }
                    obj
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&json)?);
            Ok(())
        }
        OutputFormat::Table => {
            let rows: Vec<ApprovalRow> = statuses
                .iter()
                .map(|s| ApprovalRow {
                    contract: s.contract_name.clone(),
                    collateral: if let Some(ref err) = s.collateral_error {
                        format!("\u{2717} RPC error: {err}")
                    } else {
                        format_allowance(s.collateral_allowance)
                    },
                    ctf: if let Some(ref err) = s.ctf_error {
                        format!("\u{2717} RPC error: {err}")
                    } else {
                        format_ctf(s.ctf_approved)
                    },
                })
                .collect();
            let table = tabled::Table::new(rows).with(Style::rounded()).to_string();
            println!("{table}");
            Ok(())
        }
    }
}

pub fn print_tx_result(step: usize, total: usize, label: &str, tx_hash: alloy::primitives::B256) {
    let hash_str = format!("{tx_hash}");
    let short = &hash_str[..10];
    println!("  [{step}/{total}] {label:<30} \u{2713} {short}\u{2026}");
}
