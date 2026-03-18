use anyhow::Result;
use clap::{Args, Subcommand};
use polymarket_client_sdk::bridge::{
    self,
    types::{DepositRequest, StatusRequest},
};

use crate::output::OutputFormat;
use crate::output::bridge::{print_deposit, print_status, print_supported_assets};
use crate::commands::bridge_verify::{format_onchain_status_message, verify_eth_transaction};

#[derive(Args)]
pub struct BridgeArgs {
    #[command(subcommand)]
    pub command: BridgeCommand,
}

#[derive(Subcommand)]
pub enum BridgeCommand {
    /// Get deposit addresses for a wallet (EVM, Solana, Bitcoin)
    Deposit {
        /// Polymarket wallet address (0x...)
        address: polymarket_client_sdk::types::Address,
    },
    /// List supported chains and tokens for deposits
    SupportedAssets,
    /// Check deposit transaction status for an address.
    ///
    /// If the Polymarket backend hasn't indexed your deposit yet,
    /// provide --tx-hash to independently verify on Ethereum.
    ///
    /// Example:
    ///   polymarket bridge status 0xBRIDGE_ADDR --tx-hash 0xYOUR_ETH_TX
    Status {
        /// Deposit address (EVM, Solana, or Bitcoin)
        address: String,

        /// (Optional) Your Ethereum transaction hash.
        /// When provided and the backend returns nothing, the CLI will
        /// independently verify the transaction on Ethereum via a public RPC,
        /// so you know whether your funds are safe or genuinely missing.
        #[arg(long, value_name = "TX_HASH")]
        tx_hash: Option<String>,
    },
}

pub async fn execute(
    client: &bridge::Client,
    args: BridgeArgs,
    output: OutputFormat,
) -> Result<()> {
    match args.command {
        BridgeCommand::Deposit { address } => {
            let request = DepositRequest::builder().address(address).build();
            let response = client.deposit(&request).await?;
            print_deposit(&response, &output)?;
        }
        BridgeCommand::SupportedAssets => {
            let response = client.supported_assets().await?;
            print_supported_assets(&response, &output)?;
        }
        BridgeCommand::Status { address, tx_hash } => {
            anyhow::ensure!(!address.trim().is_empty(), "Address cannot be empty");
            let request = StatusRequest::builder().address(&address).build();
            let response = client.status(&request).await?;

            // If backend found transactions — show normally
            if !response.transactions.is_empty() {
                print_status(&response, &output)?;
                return Ok(());
            }

            // ── FALLBACK: backend returned nothing ────────────────────────────
            eprintln!("⚠  Polymarket backend: No deposit found for this address.");

            match &tx_hash {
                None => {
                    eprintln!();
                    eprintln!("If you already sent a transaction and it was confirmed,");
                    eprintln!("the Polymarket indexer may simply be delayed.");
                    eprintln!();
                    eprintln!("To verify your funds are safe, re-run with your tx hash:");
                    eprintln!(
                        "  polymarket bridge status {} --tx-hash <YOUR_ETH_TX_HASH>",
                        address
                    );
                    eprintln!();
                    eprintln!("Find your tx hash in MetaMask → Activity,");
                    eprintln!("or search your address on https://etherscan.io");
                    std::process::exit(1);
                }
                Some(hash) => {
                    eprintln!();
                    eprintln!("🔍 Verifying transaction on Ethereum...");

                    match verify_eth_transaction(hash).await {
                        Ok(on_chain_status) => {
                            let msg = format_onchain_status_message(&on_chain_status);
                            if matches!(output, OutputFormat::Json) {
                                let json = serde_json::json!({
                                    "polymarket_backend": "not_found",
                                    "on_chain_verification": format!("{:?}", on_chain_status),
                                    "message": msg.trim(),
                                });
                                println!("{}", serde_json::to_string_pretty(&json)?);
                            } else {
                                println!("{}", msg);
                            }
                        }
                        Err(e) => {
                            eprintln!("⚠  Could not perform on-chain verification: {}", e);
                            eprintln!("   Check manually: https://etherscan.io/tx/{}", hash);
                            std::process::exit(1);
                        }
                    }
                }
            }
        }
    }
    Ok(())
}