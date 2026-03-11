use std::io::{self, BufRead, Write};
use std::str::FromStr;

use anyhow::{Context, Result};
use polymarket_client_sdk::auth::{LocalSigner, Signer as _};
use polymarket_client_sdk::types::Address;
use polymarket_client_sdk::{POLYGON, derive_proxy_wallet};
use secrecy::ExposeSecret;

use crate::config;

fn print_banner() {
    // #2E5CFF → RGB(46, 92, 255)
    let b = "\x1b[38;2;46;92;255m";
    let bold = "\x1b[1m";
    let dim = "\x1b[2m";
    let r = "\x1b[0m";

    println!();

    println!(
        "  {b}{bold}██████╗  ██████╗ ██╗  ██╗   ██╗███╗   ███╗ █████╗ ██████╗ ██╗  ██╗███████╗████████╗{r}"
    );
    println!(
        "  {b}{bold}██╔══██╗██╔═══██╗██║  ╚██╗ ██╔╝████╗ ████║██╔══██╗██╔══██╗██║ ██╔╝██╔════╝╚══██╔══╝{r}"
    );
    println!(
        "  {b}{bold}██████╔╝██║   ██║██║   ╚████╔╝ ██╔████╔██║███████║██████╔╝█████╔╝ █████╗     ██║{r}"
    );
    println!(
        "  {b}{bold}██╔═══╝ ██║   ██║██║    ╚██╔╝  ██║╚██╔╝██║██╔══██║██╔══██╗██╔═██╗ ██╔══╝     ██║{r}"
    );
    println!(
        "  {b}{bold}██║     ╚██████╔╝███████╗██║   ██║ ╚═╝ ██║██║  ██║██║  ██║██║  ██╗███████╗   ██║{r}"
    );
    println!(
        "  {b}{bold}╚═╝      ╚═════╝ ╚══════╝╚═╝   ╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝   ╚═╝{r}"
    );

    println!();

    // Box width matches logo (83 chars)
    println!(
        "  {b}╭─────────────────────────────────────────────────────────────────────────────────╮{r}"
    );
    println!(
        "  {b}│{r}               {bold}Preview{r} {dim}— use small amounts only, at your own risk.{r}               {b}│{r}"
    );
    println!(
        "  {b}╰─────────────────────────────────────────────────────────────────────────────────╯{r}"
    );

    println!();
}

fn prompt(msg: &str) -> Result<String> {
    print!("{msg}");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().lock().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn prompt_yn(msg: &str, default: bool) -> Result<bool> {
    let hint = if default { "Y/n" } else { "y/N" };
    let input = prompt(&format!("{msg} [{hint}] "))?;
    Ok(match input.to_lowercase().as_str() {
        "y" | "yes" => true,
        "n" | "no" => false,
        _ => default,
    })
}

fn step_header(n: u8, total: u8, label: &str) {
    println!("  [{n}/{total}] {label}");
    println!("  {}", "─".repeat(label.len() + 6));
}

pub fn execute() -> Result<()> {
    print_banner();

    let total = 4;

    step_header(1, total, "Wallet");

    let address = if config::config_exists() || config::keystore_exists() {
        // Try plaintext config / env var / flag first, then encrypted keystore
        let (existing_addr, source) = {
            let (key, src) = config::resolve_key(None);
            let addr = key
                .as_ref()
                .and_then(|k| LocalSigner::from_str(k.expose_secret()).ok())
                .map(|s| s.address());
            (addr, src)
        };
        let (existing_addr, source) = if existing_addr.is_some() {
            (existing_addr, source)
        } else if config::keystore_exists() {
            let addr = crate::password::prompt_password_with_retries(|pw| {
                config::load_key_encrypted(pw)
            })
            .ok()
            .and_then(|k| LocalSigner::from_str(k.expose_secret()).ok())
            .map(|s| s.address());
            (addr, config::KeySource::Keystore)
        } else {
            (None, source)
        };

        if let Some(addr) = existing_addr {
            println!("  ✓ Wallet already configured ({})", source.label());
            println!("    Address: {addr}");
            println!();

            if !prompt_yn("  Reconfigure wallet?", false)? {
                finish_setup(addr)?;
                return Ok(());
            }
            println!();
        }
        setup_wallet()?
    } else {
        setup_wallet()?
    };

    println!();

    finish_setup(address)
}

fn setup_wallet() -> Result<Address> {
    let has_key = prompt_yn("  Do you have an existing private key?", false)?;

    let (address, key_hex) = if has_key {
        let key = prompt("  Enter private key: ")?;
        let signer = LocalSigner::from_str(&key)
            .context("Invalid private key")?
            .with_chain_id(Some(POLYGON));
        (signer.address(), config::key_bytes_to_hex(&signer.credential().to_bytes()))
    } else {
        let signer = LocalSigner::random().with_chain_id(Some(POLYGON));
        let address = signer.address();
        (address, config::key_bytes_to_hex(&signer.credential().to_bytes()))
    };

    let password = crate::password::prompt_new_password()?;
    config::save_key_encrypted(&key_hex, &password)?;
    config::save_wallet_settings(POLYGON, config::DEFAULT_SIGNATURE_TYPE)?;

    if has_key {
        println!("  ✓ Wallet imported");
    } else {
        println!("  ✓ Wallet created");
    }
    println!("    Address: {address}");
    println!("    Config:  {}", config::config_path()?.display());

    if !has_key {
        println!();
        println!("  ⚠ Remember your password. Use `polymarket wallet export` to back up your key.");
        println!("    If lost, your funds cannot be recovered.");
    }

    Ok(address)
}

fn finish_setup(address: Address) -> Result<()> {
    let total = 4;

    step_header(2, total, "Proxy Wallet");

    let proxy = derive_proxy_wallet(address, POLYGON);
    match proxy {
        Some(proxy) => {
            println!("  ✓ Proxy wallet derived");
            println!("    Proxy: {proxy}");
            println!("    Deposit USDC to this address to start trading.");
        }
        None => {
            println!("  ✗ Could not derive proxy wallet");
            println!("    You may need to use --signature-type eoa");
        }
    }

    println!();

    step_header(3, total, "Fund Wallet");

    let deposit_addr = proxy.unwrap_or(address);
    println!("  ○ Deposit USDC to your wallet to start trading");
    println!("    Run: polymarket bridge deposit {deposit_addr}");
    println!("    Or transfer USDC directly on Polygon");

    println!();

    step_header(4, total, "Approve Contracts");

    println!("  Run `polymarket approve set` to approve contracts for trading.");
    println!("  Or `polymarket approve check` to see current approval status.");

    println!();
    println!("  ────────────────────────────────────");
    println!("  ✓ Setup complete! You're ready to go.");
    println!();
    println!("  Next steps:");
    println!("    polymarket shell              Interactive mode");
    println!("    polymarket markets list        Browse markets");
    println!("    polymarket clob book <token>   View order book");
    println!();

    Ok(())
}
