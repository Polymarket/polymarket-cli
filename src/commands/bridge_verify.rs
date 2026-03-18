/// src/commands/bridge_verify.rs
///
/// On-chain fallback verification for bridge deposits.
/// Uses Cloudflare's free public Ethereum JSON-RPC gateway —
/// no API key required.
///
/// Called by bridge.rs when the Polymarket backend returns nothing,
/// so users know whether their funds are safe on-chain or genuinely lost.

use anyhow::{anyhow, Result};
use serde_json::Value;

// ── Public result type ────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum OnChainStatus {
    /// Transaction found and succeeded on-chain
    Confirmed {
        tx_hash: String,
        from: String,
        to: String,
        confirmations: u64,
        block_number: u64,
    },
    /// Transaction found but reverted on-chain
    Failed {
        tx_hash: String,
    },
    /// Not found — pending, wrong hash, or not yet mined
    NotFound,
    /// Could not reach the RPC endpoint
    CheckFailed(String),
}

// Cloudflare's free public Ethereum mainnet RPC — no auth needed
const ETH_RPC: &str = "https://eth.llamarpc.com";

// ── Core verification logic ───────────────────────────────────────────────────

/// Verify an Ethereum transaction hash on-chain.
///
/// # Arguments
/// * `tx_hash` – 0x-prefixed Ethereum transaction hash (66 chars)
pub async fn verify_eth_transaction(tx_hash: &str) -> Result<OnChainStatus> {
    let tx_hash = tx_hash.trim();

    // Basic format guard before hitting the network
    if !tx_hash.starts_with("0x") || tx_hash.len() != 66 {
        return Err(anyhow!(
            "Invalid tx hash: expected 0x-prefixed 64-char hex, got '{}'",
            tx_hash
        ));
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| anyhow!("Failed to build HTTP client: {}", e))?;

    // Step 1: eth_getTransactionReceipt — tells us if mined + whether it succeeded
    let receipt = rpc_call(
        &client,
        ETH_RPC,
        "eth_getTransactionReceipt",
        serde_json::json!([tx_hash]),
    )
    .await?;

    if receipt["result"].is_null() {
        // Not mined yet — check if it's at least in the mempool
        let tx = rpc_call(
            &client,
            ETH_RPC,
            "eth_getTransactionByHash",
            serde_json::json!([tx_hash]),
        )
        .await?;

        if tx["result"].is_null() {
            return Ok(OnChainStatus::NotFound);
        }
        // In mempool but not mined
        return Ok(OnChainStatus::NotFound);
    }

    let result = &receipt["result"];

    // "0x1" = success, "0x0" = revert
    let status_hex = result["status"].as_str().unwrap_or("0x0");
    if status_hex == "0x0" {
        return Ok(OnChainStatus::Failed {
            tx_hash: tx_hash.to_string(),
        });
    }

    // Parse block number
    let block_hex = result["blockNumber"]
        .as_str()
        .unwrap_or("0x0")
        .trim_start_matches("0x");
    let block_number = u64::from_str_radix(block_hex, 16).unwrap_or(0);

    // Step 2: Get current block number to compute confirmations
    let block_resp = rpc_call(
        &client,
        ETH_RPC,
        "eth_blockNumber",
        serde_json::json!([]),
    )
    .await
    .unwrap_or_default();

    let current_hex = block_resp["result"]
        .as_str()
        .unwrap_or("0x0")
        .trim_start_matches("0x");
    let current_block = u64::from_str_radix(current_hex, 16).unwrap_or(block_number);
    let confirmations = current_block.saturating_sub(block_number).saturating_add(1);

    // Step 3: Get from/to addresses
    let tx_resp = rpc_call(
        &client,
        ETH_RPC,
        "eth_getTransactionByHash",
        serde_json::json!([tx_hash]),
    )
    .await
    .unwrap_or_default();

    let from = tx_resp["result"]["from"]
        .as_str()
        .unwrap_or("unknown")
        .to_string();
    let to = tx_resp["result"]["to"]
        .as_str()
        .unwrap_or("unknown")
        .to_string();

    Ok(OnChainStatus::Confirmed {
        tx_hash: tx_hash.to_string(),
        from,
        to,
        confirmations,
        block_number,
    })
}

/// Make a JSON-RPC POST call to an Ethereum node.
async fn rpc_call(
    client: &reqwest::Client,
    url: &str,
    method: &str,
    params: Value,
) -> Result<Value> {
    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1
    });

    let resp = client
        .post(url)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .map_err(|e| anyhow!("RPC request failed ({}): {}", method, e))?;

    let json: Value = resp
        .json()
        .await
        .map_err(|e| anyhow!("Failed to parse RPC response: {}", e))?;

    Ok(json)
}

// ── Display helpers ───────────────────────────────────────────────────────────

/// Format the on-chain status into a user-facing message.
pub fn format_onchain_status_message(status: &OnChainStatus) -> String {
    match status {
        OnChainStatus::Confirmed {
            tx_hash,
            from,
            to,
            confirmations,
            block_number,
        } => {
            let short = format!("{}...{}", &tx_hash[..10], &tx_hash[tx_hash.len() - 6..]);
            format!(
                "\n✅ ON-CHAIN: Transaction confirmed\n\
                 ─────────────────────────────────────────────\n\
                   Tx Hash      : {}\n\
                   From         : {}\n\
                   To (Bridge)  : {}\n\
                   Block        : {}\n\
                   Confirmations: {}\n\
                 ─────────────────────────────────────────────\n\
                 ⚠  Polymarket's backend hasn't indexed this yet.\n\
                    Your funds are safe. Wait a few minutes and retry:\n\
                    polymarket bridge status <deposit_address>",
                short, from, to, block_number, confirmations
            )
        }
        OnChainStatus::Failed { tx_hash } => {
            format!(
                "\n❌ ON-CHAIN: Transaction reverted on Ethereum\n\
                 ─────────────────────────────────────────────\n\
                   Tx Hash : {}\n\
                 ─────────────────────────────────────────────\n\
                 The transaction failed. Your USDC was NOT sent to the bridge.\n\
                 Check details: https://etherscan.io/tx/{}",
                tx_hash, tx_hash
            )
        }
        OnChainStatus::NotFound => {
            "\n⚠  ON-CHAIN: Transaction not found on Ethereum mainnet.\n\
             Possible reasons:\n\
             • The tx hash is incorrect\n\
             • Transaction is still pending (not yet mined)\n\
             • Transaction was dropped from the mempool\n\
             \nVerify at: https://etherscan.io"
                .to_string()
        }
        OnChainStatus::CheckFailed(msg) => {
            format!(
                "\n⚠  Could not verify on-chain (RPC unavailable): {}\n\
                 Check manually at: https://etherscan.io",
                msg
            )
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_short_hash() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        assert!(rt.block_on(verify_eth_transaction("0xabc")).is_err());
    }

    #[test]
    fn rejects_missing_0x_prefix() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        assert!(rt
            .block_on(verify_eth_transaction(
                "8bc4f8dabb0ce7501b672fdd4f4e8ce10a11a283b87e07e1999d2e3ba25db2cd"
            ))
            .is_err());
    }

    #[test]
    fn formats_confirmed_message() {
        let s = OnChainStatus::Confirmed {
            tx_hash: "0x8bc4f8dabb0ce7501b672fdd4f4e8ce10a11a283b87e07e1999d2e3ba25db2cd"
                .to_string(),
            from: "0xccF043006966138650aB41Ee11931201883C8DDe".to_string(),
            to: "0x892472Bd69Fa0e842869C169796c061Ea9091ED8".to_string(),
            confirmations: 109,
            block_number: 21_500_000,
        };
        let msg = format_onchain_status_message(&s);
        assert!(msg.contains("confirmed"));
        assert!(msg.contains("109"));
        assert!(msg.contains("backend hasn't indexed"));
    }

    #[test]
    fn formats_failed_message() {
        let s = OnChainStatus::Failed {
            tx_hash: "0xdeadbeef00000000000000000000000000000000000000000000000000000000"
                .to_string(),
        };
        let msg = format_onchain_status_message(&s);
        assert!(msg.contains("reverted"));
    }
}