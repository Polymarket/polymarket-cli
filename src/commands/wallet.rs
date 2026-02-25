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
    /// Delete all config and keys (fresh install)
    Reset {
        /// Skip confirmation prompt
        #[arg(long)]
        force: bool,
    },
    /// Export the private key (decrypts keystore, prints to stdout)
    Export,
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
        WalletCommand::Reset { force } => cmd_reset(output, force),
        WalletCommand::Export => cmd_export(output),
    }
}

fn guard_overwrite(force: bool) -> Result<()> {
    if !force && (config::config_exists() || config::keystore_exists()) {
        bail!(
            "A wallet already exists. Use --force to overwrite.",
        );
    }
    Ok(())
}

pub(crate) fn normalize_key(key: &str) -> String {
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

    let password = crate::password::prompt_new_password()?;
    config::save_key_encrypted(&key_hex, &password)?;
    config::save_wallet_settings(POLYGON, signature_type)?;
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
            println!("IMPORTANT: Remember your password. Use `polymarket wallet export`");
            println!("           to back up your private key. If lost, funds cannot be recovered.");
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

    let password = crate::password::prompt_new_password()?;
    config::save_key_encrypted(&normalized, &password)?;
    config::save_wallet_settings(POLYGON, signature_type)?;
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
    let key = crate::auth::resolve_key_string(private_key_flag)?;

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
    let (key_result, source) = {
        let (old_key, old_source) = config::resolve_key(private_key_flag);
        if old_key.as_ref().is_some_and(|k| !k.is_empty()) {
            (Ok(old_key.unwrap()), old_source)
        } else if config::keystore_exists() {
            let result = crate::password::prompt_password_with_retries(|pw| {
                config::load_key_encrypted(pw)
            });
            (result, config::KeySource::Keystore)
        } else {
            (Err(anyhow::anyhow!("not configured")), config::KeySource::None)
        }
    };

    let signer = key_result.ok().and_then(|k| LocalSigner::from_str(&k).ok());
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

fn cmd_export(output: &OutputFormat) -> Result<()> {
    if !config::keystore_exists() {
        bail!("{}", config::NO_WALLET_MSG);
    }

    let key = crate::password::prompt_password_with_retries(|pw| {
        config::load_key_encrypted(pw)
    })?;

    match output {
        OutputFormat::Json => {
            println!("{}", serde_json::json!({"private_key": key}));
        }
        OutputFormat::Table => {
            println!("{key}");
        }
    }
    Ok(())
}

fn cmd_reset(output: &OutputFormat, force: bool) -> Result<()> {
    if !config::config_exists() && !config::keystore_exists() {
        match output {
            OutputFormat::Table => println!("Nothing to reset. No config found."),
            OutputFormat::Json => {
                println!(
                    "{}",
                    serde_json::json!({"reset": false, "reason": "no config found"})
                );
            }
        }
        return Ok(());
    }

    if !force {
        use std::io::{self, BufRead, Write};
        print!("This will delete all keys and config. Are you sure? [y/N] ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().lock().read_line(&mut input)?;
        if !matches!(input.trim().to_lowercase().as_str(), "y" | "yes") {
            println!("Aborted.");
            return Ok(());
        }
    }

    let path = config::config_path()?;
    config::delete_config()?;

    match output {
        OutputFormat::Table => {
            println!("Config deleted: {}", path.display());
            println!("All keys and settings have been removed.");
        }
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::json!({
                    "reset": true,
                    "deleted": path.display().to_string(),
                })
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_key_adds_prefix() {
        assert_eq!(
            normalize_key("abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"),
            "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
        );
    }

    #[test]
    fn normalize_key_with_prefix_unchanged() {
        let key = "0xabcdef";
        assert_eq!(normalize_key(key), key);
    }

    #[test]
    fn normalize_key_uppercase_prefix() {
        let key = "0Xabcdef";
        assert_eq!(normalize_key(key), key);
    }
}
