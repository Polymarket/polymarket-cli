use std::str::FromStr;

use anyhow::{Context, Result, bail};
use clap::{Args, Subcommand};
use polymarket_client_sdk::auth::LocalSigner;
use polymarket_client_sdk::auth::Signer as _;
use polymarket_client_sdk::{POLYGON, derive_proxy_wallet};

use crate::config::{self, SignerType};
use crate::cwp;
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
    /// Connect an external wallet via CWP (CLI Wallet Protocol)
    Connect {
        /// CWP provider binary name (e.g. "walletconnect"); auto-discovers if omitted
        provider: Option<String>,
        /// Overwrite existing wallet
        #[arg(long)]
        force: bool,
        /// Signature type: eoa, proxy (default), or gnosis-safe
        #[arg(long, default_value = "proxy")]
        signature_type: String,
    },
    /// Disconnect the CWP wallet
    Disconnect,
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
        } => cmd_create(output, force, &signature_type),
        WalletCommand::Import {
            key,
            force,
            signature_type,
        } => cmd_import(&key, output, force, &signature_type),
        WalletCommand::Address => cmd_address(output, private_key_flag),
        WalletCommand::Show => cmd_show(output, private_key_flag),
        WalletCommand::Connect {
            provider,
            force,
            signature_type,
        } => cmd_connect(output, provider.as_deref(), force, &signature_type),
        WalletCommand::Disconnect => cmd_disconnect(output),
        WalletCommand::Reset { force } => cmd_reset(output, force),
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

fn cmd_create(output: OutputFormat, force: bool, signature_type: &str) -> Result<()> {
    guard_overwrite(force)?;

    let signer = LocalSigner::random().with_chain_id(Some(POLYGON));
    let address = signer.address();
    let key_hex = format!("{:#x}", signer.to_bytes());

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

fn cmd_import(key: &str, output: OutputFormat, force: bool, signature_type: &str) -> Result<()> {
    guard_overwrite(force)?;

    let signer = LocalSigner::from_str(key)
        .context("Invalid private key")?
        .with_chain_id(Some(POLYGON));
    let address = signer.address();
    let key_hex = format!("{:#x}", signer.to_bytes());

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

fn cmd_address(output: OutputFormat, private_key_flag: Option<&str>) -> Result<()> {
    let (key, _) = config::resolve_key(private_key_flag)?;

    // Try local key first
    if let Some(key) = key {
        let signer = LocalSigner::from_str(&key).context("Invalid private key")?;
        let address = signer.address();
        match output {
            OutputFormat::Json => {
                println!("{}", serde_json::json!({"address": address.to_string()}));
            }
            OutputFormat::Table => println!("{address}"),
        }
        return Ok(());
    }

    // Check CWP config
    if let Some(cfg) = config::load_config()? {
        if cfg.signer_type == SignerType::Cwp {
            if let Some(addr) = &cfg.cwp_address {
                match output {
                    OutputFormat::Json => {
                        println!("{}", serde_json::json!({"address": addr}));
                    }
                    OutputFormat::Table => println!("{addr}"),
                }
                return Ok(());
            }
        }
    }

    bail!("{}", config::NO_WALLET_MSG)
}

fn cmd_show(output: OutputFormat, private_key_flag: Option<&str>) -> Result<()> {
    let (key, source) = config::resolve_key(private_key_flag)?;
    let cfg = config::load_config()?;
    let is_cwp = cfg
        .as_ref()
        .is_some_and(|c| c.signer_type == SignerType::Cwp);

    let (address, proxy_addr, signer_label) = if is_cwp {
        let cfg = cfg.as_ref().unwrap();
        let addr = cfg.cwp_address.clone();
        let proxy = addr
            .as_deref()
            .and_then(|a| a.parse().ok())
            .and_then(|a| derive_proxy_wallet(a, POLYGON))
            .map(|a| a.to_string());
        let provider = cfg.cwp_provider.as_deref().unwrap_or("unknown");
        (addr, proxy, format!("cwp ({provider})"))
    } else {
        let signer = key.as_deref().and_then(|k| LocalSigner::from_str(k).ok());
        let addr = signer.as_ref().map(|s| s.address().to_string());
        let proxy = signer
            .as_ref()
            .and_then(|s| derive_proxy_wallet(s.address(), POLYGON))
            .map(|a| a.to_string());
        (addr, proxy, source.label().to_string())
    };

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
                    "signer_type": if is_cwp { "cwp" } else { "local" },
                    "config_path": config_path.display().to_string(),
                    "source": signer_label,
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
            println!("Signer type:    {}", if is_cwp { "cwp" } else { "local" });
            println!("Signature type: {sig_type}");
            println!("Config path:    {}", config_path.display());
            println!("Key source:     {signer_label}");
        }
    }
    Ok(())
}

fn cmd_connect(
    output: OutputFormat,
    provider: Option<&str>,
    force: bool,
    signature_type: &str,
) -> Result<()> {
    guard_overwrite(force)?;

    let binary = match provider {
        Some(p) => {
            // If user gave a short name, prefix with "wallet-"
            if p.starts_with("wallet-") {
                p.to_string()
            } else {
                format!("wallet-{p}")
            }
        }
        None => {
            let providers = cwp::discover();
            match providers.len() {
                0 => bail!(
                    "No CWP wallet providers found on PATH.\n\
                     Install a wallet-* binary (e.g. wallet-walletconnect) and try again."
                ),
                1 => {
                    let p = &providers[0];
                    eprintln!("Discovered CWP provider: {} ({})", p.name, p.binary);
                    p.binary.clone()
                }
                _ => {
                    eprintln!("Multiple CWP providers found:");
                    for (i, p) in providers.iter().enumerate() {
                        eprintln!("  {}. {} ({})", i + 1, p.name, p.binary);
                    }
                    bail!(
                        "Multiple providers found. Specify one with: polymarket wallet connect <provider>"
                    );
                }
            }
        }
    };

    let signer = cwp::connect(&binary).context("Failed to connect via CWP")?;
    let address = alloy::signers::Signer::address(&signer);
    let proxy_addr = derive_proxy_wallet(address, POLYGON);

    config::save_cwp_wallet(&binary, &address.to_string(), POLYGON, signature_type)?;
    let config_path = config::config_path()?;

    match output {
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::json!({
                    "address": address.to_string(),
                    "proxy_address": proxy_addr.map(|a| a.to_string()),
                    "provider": binary,
                    "signature_type": signature_type,
                    "config_path": config_path.display().to_string(),
                })
            );
        }
        OutputFormat::Table => {
            println!("Wallet connected via CWP!");
            println!("Address:        {address}");
            if let Some(proxy) = proxy_addr {
                println!("Proxy wallet:   {proxy}");
            }
            println!("Provider:       {binary}");
            println!("Signature type: {signature_type}");
            println!("Config:         {}", config_path.display());
        }
    }
    Ok(())
}

fn cmd_disconnect(output: OutputFormat) -> Result<()> {
    if let Some(cfg) = config::load_config()? {
        if cfg.signer_type != SignerType::Cwp {
            match output {
                OutputFormat::Table => println!("No CWP wallet connected. Nothing to disconnect."),
                OutputFormat::Json => {
                    println!(
                        "{}",
                        serde_json::json!({"disconnected": false, "reason": "not a cwp wallet"})
                    );
                }
            }
            return Ok(());
        }
    } else {
        match output {
            OutputFormat::Table => println!("No wallet configured. Nothing to disconnect."),
            OutputFormat::Json => {
                println!(
                    "{}",
                    serde_json::json!({"disconnected": false, "reason": "no config"})
                );
            }
        }
        return Ok(());
    }

    let path = config::config_path()?;
    config::delete_config()?;

    match output {
        OutputFormat::Table => {
            println!("CWP wallet disconnected.");
            println!("Config deleted: {}", path.display());
        }
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::json!({
                    "disconnected": true,
                    "deleted": path.display().to_string(),
                })
            );
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
