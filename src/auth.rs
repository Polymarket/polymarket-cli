use std::str::FromStr;

use alloy::primitives::B256;
use alloy::providers::{Provider, ProviderBuilder};
use anyhow::{Context, Result, bail};
use polymarket_client_sdk::auth::state::Authenticated;
use polymarket_client_sdk::auth::{LocalSigner, Normal, Signer as _};
use polymarket_client_sdk::clob::types::SignatureType;
use polymarket_client_sdk::types::Address;
use polymarket_client_sdk::{POLYGON, clob};

use crate::config::{self, SignerType};
use crate::cwp::{CwpSigner, PolySigner};

const DEFAULT_RPC_URL: &str = "https://polygon.drpc.org";

fn rpc_url() -> String {
    std::env::var("POLYMARKET_RPC_URL").unwrap_or_else(|_| DEFAULT_RPC_URL.to_string())
}

fn parse_signature_type(s: &str) -> SignatureType {
    match s {
        config::DEFAULT_SIGNATURE_TYPE => SignatureType::Proxy,
        "gnosis-safe" => SignatureType::GnosisSafe,
        _ => SignatureType::Eoa,
    }
}

pub fn resolve_signer(private_key: Option<&str>) -> Result<PolySigner> {
    // CLI flag or env var always uses local signer
    let (key, _source) = config::resolve_key(private_key)?;
    if let Some(key) = key {
        let signer = LocalSigner::from_str(&key)
            .context("Invalid private key")?
            .with_chain_id(Some(POLYGON));
        return Ok(PolySigner::Local(signer));
    }

    // Check config for CWP
    if let Some(cfg) = config::load_config()? {
        if cfg.signer_type == SignerType::Cwp {
            let provider = cfg
                .cwp_provider
                .context("CWP provider not configured")?;
            let address: alloy::primitives::Address = cfg
                .cwp_address
                .context("CWP address not configured")?
                .parse()
                .context("Invalid CWP address in config")?;
            let signer = CwpSigner::new(&provider, address, Some(POLYGON));
            return Ok(PolySigner::Cwp(signer));
        }
    }

    bail!("{}", config::NO_WALLET_MSG)
}

pub async fn authenticated_clob_client(
    private_key: Option<&str>,
    signature_type_flag: Option<&str>,
) -> Result<clob::Client<Authenticated<Normal>>> {
    let signer = resolve_signer(private_key)?;
    authenticate_with_signer(&signer, signature_type_flag).await
}

pub async fn authenticate_with_signer(
    signer: &(impl polymarket_client_sdk::auth::Signer + Sync),
    signature_type_flag: Option<&str>,
) -> Result<clob::Client<Authenticated<Normal>>> {
    let sig_type = parse_signature_type(&config::resolve_signature_type(signature_type_flag)?);

    clob::Client::default()
        .authentication_builder(signer)
        .signature_type(sig_type)
        .authenticate()
        .await
        .context("Failed to authenticate with Polymarket CLOB")
}

pub async fn create_readonly_provider() -> Result<impl alloy::providers::Provider + Clone> {
    ProviderBuilder::new()
        .connect(&rpc_url())
        .await
        .context("Failed to connect to Polygon RPC")
}

pub async fn create_provider(
    private_key: Option<&str>,
) -> Result<impl alloy::providers::Provider + Clone> {
    let (key, _) = config::resolve_key(private_key)?;
    let key = key.ok_or_else(|| {
        // Check if CWP wallet is configured — give a better error message
        if let Some(cfg) = config::load_config().ok().flatten() {
            if cfg.signer_type == SignerType::Cwp {
                return anyhow::anyhow!(
                    "CTF operations require a local wallet. Use `polymarket wallet` to configure one."
                );
            }
        }
        anyhow::anyhow!("{}", config::NO_WALLET_MSG)
    })?;
    let signer = LocalSigner::from_str(&key)
        .context("Invalid private key")?
        .with_chain_id(Some(POLYGON));
    ProviderBuilder::new()
        .wallet(signer)
        .connect(&rpc_url())
        .await
        .context("Failed to connect to Polygon RPC with wallet")
}

pub async fn send_and_confirm_cwp_tx(
    cwp_signer: &CwpSigner,
    provider: &(impl Provider + Sync),
    to: Address,
    calldata: Vec<u8>,
) -> Result<B256> {
    // Send via CWP wallet (wallet handles gas estimation)
    let tx_hash = cwp_signer
        .send_transaction(to, calldata, alloy::primitives::U256::ZERO, None)
        .await
        .context("CWP send-transaction failed")?;

    // Wait for on-chain confirmation (poll every 2s, timeout at 2 min)
    for i in 0..60 {
        if i > 0 {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
        if let Some(receipt) = provider.get_transaction_receipt(tx_hash).await? {
            if !receipt.status() {
                bail!("Transaction {tx_hash} reverted on-chain");
            }
            return Ok(tx_hash);
        }
    }
    bail!("Transaction {tx_hash} not confirmed after 2 minutes")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_signature_type_proxy() {
        assert_eq!(parse_signature_type("proxy"), SignatureType::Proxy);
    }

    #[test]
    fn parse_signature_type_gnosis_safe() {
        assert_eq!(
            parse_signature_type("gnosis-safe"),
            SignatureType::GnosisSafe
        );
    }

    #[test]
    fn parse_signature_type_eoa() {
        assert_eq!(parse_signature_type("eoa"), SignatureType::Eoa);
    }

    #[test]
    fn parse_signature_type_unknown_defaults_to_eoa() {
        assert_eq!(parse_signature_type("unknown"), SignatureType::Eoa);
    }
}
