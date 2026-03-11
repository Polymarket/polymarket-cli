use alloy::primitives::U256;
use anyhow::Result;
use tabled::Tabled;
use tabled::settings::Style;

use super::OutputFormat;

pub struct ApprovalStatus {
    pub contract_name: String,
    pub contract_address: String,
    pub usdc_allowance: U256,
    pub ctf_approved: bool,
    pub usdc_error: Option<String>,
    pub ctf_error: Option<String>,
}

#[derive(Tabled)]
struct ApprovalRow {
    #[tabled(rename = "Contract")]
    contract: String,
    #[tabled(rename = "USDC")]
    usdc: String,
    #[tabled(rename = "CTF Tokens")]
    ctf: String,
}

fn format_allowance(allowance: U256) -> String {
    if allowance == U256::MAX {
        "\u{2713} Unlimited".to_string()
    } else if allowance == U256::ZERO {
        "\u{2717} None".to_string()
    } else {
        let usdc_decimals = U256::from(1_000_000);
        let whole = allowance / usdc_decimals;
        format!("\u{2713} {whole} USDC")
    }
}

fn format_ctf(approved: bool) -> String {
    if approved {
        "\u{2713} Approved".to_string()
    } else {
        "\u{2717} Not set".to_string()
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
                        "usdc_allowance": s.usdc_allowance.to_string(),
                        "usdc_approved": s.usdc_allowance > U256::ZERO,
                        "ctf_approved": s.ctf_approved,
                    });
                    if let Some(ref err) = s.usdc_error {
                        obj["usdc_error"] = serde_json::Value::String(err.clone());
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
                    usdc: if let Some(ref err) = s.usdc_error {
                        format!("\u{2717} RPC error: {err}")
                    } else {
                        format_allowance(s.usdc_allowance)
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
