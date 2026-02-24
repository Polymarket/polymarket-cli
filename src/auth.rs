use std::str::FromStr;

use alloy::primitives::{Bytes, U256};
use alloy::providers::ProviderBuilder;
use alloy::sol;
use anyhow::{Context, Result};
use polymarket_client_sdk::auth::state::Authenticated;
use polymarket_client_sdk::auth::{LocalSigner, Normal, Signer as _};
use polymarket_client_sdk::clob::types::SignatureType;
use polymarket_client_sdk::types::Address;
use polymarket_client_sdk::{POLYGON, clob, derive_proxy_wallet};

use crate::config;

pub const RPC_URL: &str = "https://polygon.drpc.org";

sol! {
    #[allow(clippy::exhaustive_structs)]
    #[sol(rpc)]
    interface IProxyWallet {
        struct ProxyCall {
            uint8 typeCode;
            address to;
            uint256 value;
            bytes data;
        }

        function proxy(ProxyCall[] memory calls) external payable returns (bytes[] memory);
    }
}

/// Returns `true` when the resolved signature type is proxy mode.
pub fn is_proxy_mode(signature_type: Option<&str>) -> bool {
    config::resolve_signature_type(signature_type) == config::DEFAULT_SIGNATURE_TYPE
}

/// Derives the proxy wallet address for the configured private key.
/// Returns `None` when not in proxy mode or when derivation fails.
pub fn resolve_proxy_address(
    private_key: Option<&str>,
    signature_type: Option<&str>,
) -> Result<Option<Address>> {
    if !is_proxy_mode(signature_type) {
        return Ok(None);
    }
    let signer = resolve_signer(private_key)?;
    let eoa = polymarket_client_sdk::auth::Signer::address(&signer);
    let proxy = derive_proxy_wallet(eoa, POLYGON)
        .ok_or_else(|| anyhow::anyhow!("Could not derive proxy wallet for {eoa}"))?;
    Ok(Some(proxy))
}

/// Sends a transaction through the proxy wallet's `proxy` function.
///
/// Wraps the call in a single-element `ProxyCall` array with `typeCode = 1`
/// (CALL) and `value = 0`, matching the on-chain ProxyWallet contract at
/// <https://github.com/Polymarket/proxy-factories>.
pub async fn proxy_exec(
    provider: &(impl alloy::providers::Provider + Clone),
    proxy_address: Address,
    target: Address,
    calldata: Bytes,
) -> Result<alloy::primitives::B256> {
    let proxy = IProxyWallet::new(proxy_address, provider);
    let call = IProxyWallet::ProxyCall {
        typeCode: 1, // CallType.CALL
        to: target,
        value: U256::ZERO,
        data: calldata,
    };
    proxy
        .proxy(vec![call])
        .send()
        .await
        .context("Failed to send proxy transaction")?
        .watch()
        .await
        .context("Failed to confirm proxy transaction")
}

fn parse_signature_type(s: &str) -> SignatureType {
    match s {
        config::DEFAULT_SIGNATURE_TYPE => SignatureType::Proxy,
        "gnosis-safe" => SignatureType::GnosisSafe,
        _ => SignatureType::Eoa,
    }
}

pub fn resolve_signer(
    private_key: Option<&str>,
) -> Result<impl polymarket_client_sdk::auth::Signer> {
    let (key, _) = config::resolve_key(private_key);
    let key = key.ok_or_else(|| anyhow::anyhow!("{}", config::NO_WALLET_MSG))?;
    LocalSigner::from_str(&key)
        .context("Invalid private key")
        .map(|s| s.with_chain_id(Some(POLYGON)))
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
    let sig_type = parse_signature_type(&config::resolve_signature_type(signature_type_flag));

    clob::Client::default()
        .authentication_builder(signer)
        .signature_type(sig_type)
        .authenticate()
        .await
        .context("Failed to authenticate with Polymarket CLOB")
}

pub async fn create_readonly_provider() -> Result<impl alloy::providers::Provider + Clone> {
    ProviderBuilder::new()
        .connect(RPC_URL)
        .await
        .context("Failed to connect to Polygon RPC")
}

pub async fn create_provider(
    private_key: Option<&str>,
) -> Result<impl alloy::providers::Provider + Clone> {
    let (key, _) = config::resolve_key(private_key);
    let key = key.ok_or_else(|| anyhow::anyhow!("{}", config::NO_WALLET_MSG))?;
    let signer = LocalSigner::from_str(&key)
        .context("Invalid private key")?
        .with_chain_id(Some(POLYGON));
    ProviderBuilder::new()
        .wallet(signer)
        .connect(RPC_URL)
        .await
        .context("Failed to connect to Polygon RPC with wallet")
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
