use std::fmt::Write as _;
use std::str::FromStr;

use anyhow::{Context, Result, bail};
use clap::{Args, Subcommand};
use polymarket_client_sdk::auth::LocalSigner;
use polymarket_client_sdk::auth::Signer as _;
use polymarket_client_sdk::{POLYGON, derive_proxy_wallet};

use crate::config;
use crate::output::OutputFormat;

#[derive(Args)]
pub struct WalletArgs {
    #[command(subcommand)]
    pub command: WalletCommand,
}

#[derive(Subcommand)]
pub enum WalletCommand {
    /// Generate a new random wallet and save to config
    Create {
        /// Overwrite existing wallet
        #[arg(long)]
        force: bool,
        /// Signature type: eoa, proxy (default), or gnosis-safe
        #[arg(long, default_value = "proxy")]
        signature_type: String,
    },
    /// Import an existing private key
    Import {
        /// Private key (hex, with or without 0x prefix)
        key: String,
        /// Overwrite existing wallet
        #[arg(long)]
        force: bool,
        /// Signature type: eoa, proxy (default), or gnosis-safe
        #[arg(long, default_value = "proxy")]
        signature_type: String,
    },
    /// Show the address of the configured wallet
    Address,
    /// Show wallet info (address, config path, key source)
    Show,
}

pub fn execute(
    args: WalletArgs,
    output: &OutputFormat,
    private_key_flag: Option<&str>,
) -> Result<()> {
    match args.command {
        WalletCommand::Create {
            force,
            signature_type,
        } => cmd_create(output, force, &signature_type),
        WalletCommand::Import {
            key,
            force,
            signature_type,
        } => cmd_import(&key, output, force, &signature_type),
        WalletCommand::Address => cmd_address(output, private_key_flag),
        WalletCommand::Show => cmd_show(output, private_key_flag),
    }
}

fn guard_overwrite(force: bool) -> Result<()> {
    if !force && config::config_exists() {
        bail!(
            "A wallet already exists at {}. Use --force to overwrite.",
            config::config_path()?.display()
        );
    }
    Ok(())
}

fn normalize_key(key: &str) -> String {
    if key.starts_with("0x") || key.starts_with("0X") {
        key.to_string()
    } else {
        format!("0x{key}")
    }
}

fn cmd_create(output: &OutputFormat, force: bool, signature_type: &str) -> Result<()> {
    guard_overwrite(force)?;

    let signer = LocalSigner::random().with_chain_id(Some(POLYGON));
    let address = signer.address();
    let bytes = signer.credential().to_bytes();
    let mut key_hex = String::with_capacity(2 + bytes.len() * 2);
    key_hex.push_str("0x");
    for b in &bytes {
        write!(key_hex, "{b:02x}").unwrap();
    }

    config::save_wallet(&key_hex, POLYGON, signature_type)?;
    let config_path = config::config_path()?;
    let proxy_addr = derive_proxy_wallet(address, POLYGON);

    match output {
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::json!({
                    "address": address.to_string(),
                    "proxy_address": proxy_addr.map(|a| a.to_string()),
                    "signature_type": signature_type,
                    "config_path": config_path.display().to_string(),
                })
            );
        }
        OutputFormat::Table => {
            println!("Wallet created successfully!");
            println!("Address:        {address}");
            if let Some(proxy) = proxy_addr {
                println!("Proxy wallet:   {proxy}");
            }
            println!("Signature type: {signature_type}");
            println!("Config:         {}", config_path.display());
            println!();
            println!("IMPORTANT: Back up your private key from the config file.");
            println!("           If lost, your funds cannot be recovered.");
        }
    }
    Ok(())
}

fn cmd_import(key: &str, output: &OutputFormat, force: bool, signature_type: &str) -> Result<()> {
    guard_overwrite(force)?;

    let normalized = normalize_key(key);
    let signer = LocalSigner::from_str(&normalized)
        .context("Invalid private key")?
        .with_chain_id(Some(POLYGON));
    let address = signer.address();

    config::save_wallet(&normalized, POLYGON, signature_type)?;
    let config_path = config::config_path()?;
    let proxy_addr = derive_proxy_wallet(address, POLYGON);

    match output {
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::json!({
                    "address": address.to_string(),
                    "proxy_address": proxy_addr.map(|a| a.to_string()),
                    "signature_type": signature_type,
                    "config_path": config_path.display().to_string(),
                })
            );
        }
        OutputFormat::Table => {
            println!("Wallet imported successfully!");
            println!("Address:        {address}");
            if let Some(proxy) = proxy_addr {
                println!("Proxy wallet:   {proxy}");
            }
            println!("Signature type: {signature_type}");
            println!("Config:         {}", config_path.display());
        }
    }
    Ok(())
}

fn cmd_address(output: &OutputFormat, private_key_flag: Option<&str>) -> Result<()> {
    let (key, _) = config::resolve_key(private_key_flag);
    let key = key.ok_or_else(|| anyhow::anyhow!("{}", config::NO_WALLET_MSG))?;

    let signer = LocalSigner::from_str(&key).context("Invalid private key")?;
    let address = signer.address();

    match output {
        OutputFormat::Json => {
            println!("{}", serde_json::json!({"address": address.to_string()}));
        }
        OutputFormat::Table => {
            println!("{address}");
        }
    }
    Ok(())
}

fn cmd_show(output: &OutputFormat, private_key_flag: Option<&str>) -> Result<()> {
    let (key, source) = config::resolve_key(private_key_flag);
    let signer = key.as_deref().and_then(|k| LocalSigner::from_str(k).ok());
    let address = signer.as_ref().map(|s| s.address().to_string());
    let proxy_addr = signer
        .as_ref()
        .and_then(|s| derive_proxy_wallet(s.address(), POLYGON))
        .map(|a| a.to_string());

    let sig_type = config::resolve_signature_type(None);
    let config_path = config::config_path()?;

    match output {
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::json!({
                    "address": address,
                    "proxy_address": proxy_addr,
                    "signature_type": sig_type,
                    "config_path": config_path.display().to_string(),
                    "source": source.label(),
                    "configured": address.is_some(),
                })
            );
        }
        OutputFormat::Table => {
            match &address {
                Some(addr) => println!("Address:        {addr}"),
                None => println!("Address:        (not configured)"),
            }
            if let Some(proxy) = &proxy_addr {
                println!("Proxy wallet:   {proxy}");
            }
            println!("Signature type: {sig_type}");
            println!("Config path:    {}", config_path.display());
            println!("Key source:     {}", source.label());
        }
    }
    Ok(())
}
