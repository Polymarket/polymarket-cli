use std::str::FromStr;

use alloy::providers::ProviderBuilder;
use anyhow::{Context, Result};
use polymarket_client_sdk::auth::state::Authenticated;
use polymarket_client_sdk::auth::{LocalSigner, Normal, Signer as _};
use polymarket_client_sdk::clob::types::SignatureType;
use polymarket_client_sdk::{POLYGON, clob};

use crate::config;

pub const RPC_URL: &str = "https://polygon.drpc.org";

fn parse_signature_type(s: &str) -> SignatureType {
    match s {
        config::DEFAULT_SIGNATURE_TYPE => SignatureType::Proxy,
        "gnosis-safe" => SignatureType::GnosisSafe,
        _ => SignatureType::Eoa,
    }
}

/// Resolve the private key hex string, prompting for password if needed.
pub(crate) fn resolve_key_string(private_key: Option<&str>) -> Result<String> {
    // Auto-migrate plaintext config to encrypted keystore
    if config::needs_migration() {
        eprintln!("Your wallet key is stored in plaintext. Encrypting it now...");
        let password = crate::password::prompt_new_password()?;
        config::migrate_to_encrypted(&password)?;
        eprintln!("Wallet key encrypted successfully.");
        return config::load_key_encrypted(&password);
    }

    // 1. CLI flag
    if let Some(key) = private_key {
        return Ok(key.to_string());
    }
    // 2. Env var
    if let Ok(key) = std::env::var(config::ENV_VAR)
        && !key.is_empty()
    {
        return Ok(key);
    }
    // 3. Old config (plaintext — for backward compat)
    if let Some(cfg) = config::load_config()
        && !cfg.private_key.is_empty()
    {
        return Ok(cfg.private_key);
    }
    // 4. Encrypted keystore with retry
    if config::keystore_exists() {
        return crate::password::prompt_password_with_retries(|pw| {
            config::load_key_encrypted(pw)
        });
    }
    anyhow::bail!("{}", config::NO_WALLET_MSG)
}

pub fn resolve_signer(
    private_key: Option<&str>,
) -> Result<impl polymarket_client_sdk::auth::Signer> {
    let key = resolve_key_string(private_key)?;
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
    let key = resolve_key_string(private_key)?;
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
