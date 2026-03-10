use std::str::FromStr;

use alloy::signers::local::PrivateKeySigner;
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
        /// Store the key as plaintext (not recommended)
        #[arg(long)]
        no_password: bool,
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
        /// Store the key as plaintext (not recommended)
        #[arg(long)]
        no_password: bool,
    },
    /// Show the address of the configured wallet
    Address,
    /// Show wallet info (address, config path, key source)
    Show,
    /// Export the private key (decrypts if encrypted)
    Export,
    /// Delete all config and keys (fresh install)
    Reset {
        /// Skip confirmation prompt
        #[arg(long)]
        force: bool,
    },
}

pub fn execute(
    args: WalletArgs,
    output: OutputFormat,
    private_key_flag: Option<&str>,
) -> Result<()> {
    match args.command {
        WalletCommand::Create {
            force,
            signature_type,
            no_password,
        } => cmd_create(output, force, &signature_type, no_password),
        WalletCommand::Import {
            key,
            force,
            signature_type,
            no_password,
        } => cmd_import(&key, output, force, &signature_type, no_password),
        WalletCommand::Address => cmd_address(output, private_key_flag),
        WalletCommand::Show => cmd_show(output, private_key_flag),
        WalletCommand::Export => cmd_export(output, private_key_flag),
        WalletCommand::Reset { force } => cmd_reset(output, force),
    }
}

fn save_encrypted(signer: &PrivateKeySigner, password: &str, signature_type: &str) -> Result<()> {
    let ks_dir = config::keystore_dir()?;
    std::fs::create_dir_all(&ks_dir).context("Failed to create keystore directory")?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&ks_dir, std::fs::Permissions::from_mode(0o700))?;
    }

    // Remove any existing keystore files
    if let Ok(entries) = std::fs::read_dir(&ks_dir) {
        for entry in entries.flatten() {
            let _ = std::fs::remove_file(entry.path());
        }
    }

    let mut rng = rand::thread_rng();
    PrivateKeySigner::encrypt_keystore(&ks_dir, &mut rng, signer.to_bytes(), password, None)
        .context("Failed to encrypt keystore")?;

    config::save_wallet_encrypted(POLYGON, signature_type)
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

fn cmd_create(
    output: OutputFormat,
    force: bool,
    signature_type: &str,
    no_password: bool,
) -> Result<()> {
    guard_overwrite(force)?;

    let signer = LocalSigner::random().with_chain_id(Some(POLYGON));
    let address = signer.address();
    let encrypted = !no_password;

    if encrypted {
        let password = config::read_new_password()?;
        save_encrypted(&signer, &password, signature_type)?;
    } else {
        let key_hex = format!("{:#x}", signer.to_bytes());
        config::save_wallet(&key_hex, POLYGON, signature_type)?;
    }

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
                    "encrypted": encrypted,
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
            println!("Encrypted:      {encrypted}");
            println!("Config:         {}", config_path.display());
            if !encrypted {
                println!();
                println!(
                    "WARNING: Key stored as plaintext. Use without --no-password for encryption."
                );
            }
        }
    }
    Ok(())
}

fn cmd_import(
    key: &str,
    output: OutputFormat,
    force: bool,
    signature_type: &str,
    no_password: bool,
) -> Result<()> {
    guard_overwrite(force)?;

    let signer = LocalSigner::from_str(key)
        .context("Invalid private key")?
        .with_chain_id(Some(POLYGON));
    let address = signer.address();
    let encrypted = !no_password;

    if encrypted {
        let password = config::read_new_password()?;
        save_encrypted(&signer, &password, signature_type)?;
    } else {
        let key_hex = format!("{:#x}", signer.to_bytes());
        config::save_wallet(&key_hex, POLYGON, signature_type)?;
    }

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
                    "encrypted": encrypted,
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
            println!("Encrypted:      {encrypted}");
            println!("Config:         {}", config_path.display());
        }
    }
    Ok(())
}

fn cmd_address(output: OutputFormat, private_key_flag: Option<&str>) -> Result<()> {
    let (key, _) = config::resolve_key(private_key_flag)?;
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

fn cmd_show(output: OutputFormat, private_key_flag: Option<&str>) -> Result<()> {
    let (key, source) = config::resolve_key(private_key_flag)?;
    let signer = key.as_deref().and_then(|k| LocalSigner::from_str(k).ok());
    let address = signer.as_ref().map(|s| s.address().to_string());
    let proxy_addr = signer
        .as_ref()
        .and_then(|s| derive_proxy_wallet(s.address(), POLYGON))
        .map(|a| a.to_string());

    let sig_type = config::resolve_signature_type(None)?;
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

fn cmd_export(output: OutputFormat, private_key_flag: Option<&str>) -> Result<()> {
    let (key, source) = config::resolve_key(private_key_flag)?;
    let key = key.ok_or_else(|| anyhow::anyhow!("{}", config::NO_WALLET_MSG))?;

    match output {
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::json!({
                    "private_key": key,
                    "source": source.label(),
                })
            );
        }
        OutputFormat::Table => {
            println!("Private key: {key}");
            println!("Source:      {}", source.label());
            println!();
            println!("WARNING: Do not share this key. Anyone with it can access your funds.");
        }
    }
    Ok(())
}

fn cmd_reset(output: OutputFormat, force: bool) -> Result<()> {
    if !config::config_exists() {
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
