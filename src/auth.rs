use std::str::FromStr;

use alloy::providers::ProviderBuilder;
use anyhow::{Context, Result};
use polymarket_client_sdk_v2::auth::state::Authenticated;
use polymarket_client_sdk_v2::auth::{LocalSigner, Normal, Signer as _};
use polymarket_client_sdk_v2::clob::types::SignatureType;
use polymarket_client_sdk_v2::{POLYGON, clob};

use crate::config;

const DEFAULT_CLOB_HOST: &str = "https://clob.polymarket.com";
const DEFAULT_RPC_URL: &str = "https://polygon.drpc.org";

fn clob_host() -> String {
    std::env::var("POLYMARKET_CLOB_HOST").unwrap_or_else(|_| DEFAULT_CLOB_HOST.to_string())
}

fn rpc_url() -> String {
    std::env::var("POLYMARKET_RPC_URL").unwrap_or_else(|_| DEFAULT_RPC_URL.to_string())
}

fn parse_signature_type(s: &str) -> Result<SignatureType> {
    match s {
        config::DEFAULT_SIGNATURE_TYPE => Ok(SignatureType::Proxy),
        "gnosis-safe" => Ok(SignatureType::GnosisSafe),
        config::POLY_1271_SIGNATURE_TYPE => Ok(SignatureType::Poly1271),
        "eoa" => Ok(SignatureType::Eoa),
        _ => Err(anyhow::anyhow!("Unsupported signature type: {s}")),
    }
}

pub fn resolve_signer(
    private_key: Option<&str>,
) -> Result<impl polymarket_client_sdk_v2::auth::Signer> {
    let (key, _) = config::resolve_key(private_key)?;
    let key = key.ok_or_else(|| anyhow::anyhow!("{}", config::NO_WALLET_MSG))?;
    LocalSigner::from_str(&key)
        .context("Invalid private key")
        .map(|s| s.with_chain_id(Some(POLYGON)))
}

pub async fn authenticated_clob_client(
    private_key: Option<&str>,
    signature_type_flag: Option<&str>,
    funder_flag: Option<&str>,
) -> Result<clob::Client<Authenticated<Normal>>> {
    let signer = resolve_signer(private_key)?;
    authenticate_with_signer(&signer, signature_type_flag, funder_flag).await
}

pub async fn authenticate_with_signer(
    signer: &(impl polymarket_client_sdk_v2::auth::Signer + Sync),
    signature_type_flag: Option<&str>,
    funder_flag: Option<&str>,
) -> Result<clob::Client<Authenticated<Normal>>> {
    let signature_type = config::resolve_signature_type(signature_type_flag)?;
    let sig_type = parse_signature_type(&signature_type)?;
    let funder = config::validate_funder_for_signature_type(
        &signature_type,
        config::resolve_funder(funder_flag)?,
    )?;

    let mut builder = unauthenticated_clob_client()?
        .authentication_builder(signer)
        .signature_type(sig_type);
    if let Some(funder) = funder {
        builder = builder.funder(funder);
    }

    builder
        .authenticate()
        .await
        .context("Failed to authenticate with Polymarket CLOB")
}

pub fn unauthenticated_clob_client() -> Result<clob::Client> {
    clob::Client::new(&clob_host(), clob::Config::default())
        .context("Failed to create Polymarket CLOB client")
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
    let key = key.ok_or_else(|| anyhow::anyhow!("{}", config::NO_WALLET_MSG))?;
    let signer = LocalSigner::from_str(&key)
        .context("Invalid private key")?
        .with_chain_id(Some(POLYGON));
    ProviderBuilder::new()
        .wallet(signer)
        .connect(&rpc_url())
        .await
        .context("Failed to connect to Polygon RPC with wallet")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_signature_type_proxy() {
        assert_eq!(parse_signature_type("proxy").unwrap(), SignatureType::Proxy);
    }

    #[test]
    fn parse_signature_type_gnosis_safe() {
        assert_eq!(
            parse_signature_type("gnosis-safe").unwrap(),
            SignatureType::GnosisSafe
        );
    }

    #[test]
    fn parse_signature_type_eoa() {
        assert_eq!(parse_signature_type("eoa").unwrap(), SignatureType::Eoa);
    }

    #[test]
    fn parse_signature_type_poly_1271() {
        assert_eq!(
            parse_signature_type("poly-1271").unwrap(),
            SignatureType::Poly1271
        );
    }

    #[test]
    fn parse_signature_type_unknown_is_rejected() {
        assert!(parse_signature_type("unknown").is_err());
    }
}
