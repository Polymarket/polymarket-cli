use alloy::primitives::U256;
use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use polymarket_client_sdk::ctf::types::{
    CollectionIdRequest, ConditionIdRequest, MergePositionsRequest, PositionIdRequest,
    RedeemNegRiskRequest, RedeemPositionsRequest, SplitPositionRequest,
};
use polymarket_client_sdk::types::{Address, B256};
use polymarket_client_sdk::{POLYGON, ctf};
use rust_decimal::Decimal;

use crate::auth;
use crate::output::OutputFormat;
use crate::output::ctf as ctf_output;

const USDC_DECIMALS: Decimal = Decimal::from_parts(1_000_000, 0, 0, false, 0);

use super::USDC_ADDRESS_STR;

#[derive(Args)]
pub struct CtfArgs {
    #[command(subcommand)]
    pub command: CtfCommand,
}

#[derive(Subcommand)]
pub enum CtfCommand {
    /// Split collateral into outcome tokens
    Split {
        /// Condition ID (0x-prefixed 32-byte hex)
        #[arg(long)]
        condition: B256,
        /// Amount in USDC (e.g. 10 for $10)
        #[arg(long)]
        amount: String,
        /// Collateral token address (defaults to USDC)
        #[arg(long, default_value = USDC_ADDRESS_STR)]
        collateral: Address,
        /// Custom partition as comma-separated index sets (e.g. "1,2" for binary, "1,2,4" for 3-outcome)
        #[arg(long)]
        partition: Option<String>,
        /// Parent collection ID for nested positions (defaults to zero)
        #[arg(long)]
        parent_collection: Option<B256>,
    },
    /// Merge outcome tokens back into collateral
    Merge {
        /// Condition ID (0x-prefixed 32-byte hex)
        #[arg(long)]
        condition: B256,
        /// Amount in USDC (e.g. 10 for $10)
        #[arg(long)]
        amount: String,
        /// Collateral token address (defaults to USDC)
        #[arg(long, default_value = USDC_ADDRESS_STR)]
        collateral: Address,
        /// Custom partition as comma-separated index sets (e.g. "1,2" for binary, "1,2,4" for 3-outcome)
        #[arg(long)]
        partition: Option<String>,
        /// Parent collection ID for nested positions (defaults to zero)
        #[arg(long)]
        parent_collection: Option<B256>,
    },
    /// Redeem winning tokens after market resolution
    Redeem {
        /// Condition ID (0x-prefixed 32-byte hex)
        #[arg(long)]
        condition: B256,
        /// Collateral token address (defaults to USDC)
        #[arg(long, default_value = USDC_ADDRESS_STR)]
        collateral: Address,
        /// Custom index sets as comma-separated values (e.g. "1,2" for binary, "1" for YES only)
        #[arg(long)]
        index_sets: Option<String>,
        /// Parent collection ID for nested positions (defaults to zero)
        #[arg(long)]
        parent_collection: Option<B256>,
    },
    /// Redeem neg-risk positions
    RedeemNegRisk {
        /// Condition ID (0x-prefixed 32-byte hex)
        #[arg(long)]
        condition: B256,
        /// Comma-separated amounts in USDC for each outcome (e.g. "10,5")
        #[arg(long)]
        amounts: String,
    },
    /// Calculate a condition ID from oracle, question, and outcome count
    ConditionId {
        /// Oracle address (0x-prefixed)
        #[arg(long)]
        oracle: Address,
        /// Question ID (0x-prefixed 32-byte hex)
        #[arg(long)]
        question: B256,
        /// Number of outcomes (e.g. 2 for binary)
        #[arg(long)]
        outcomes: u64,
    },
    /// Calculate a collection ID from condition and index set
    CollectionId {
        /// Condition ID (0x-prefixed 32-byte hex)
        #[arg(long)]
        condition: B256,
        /// Index set (e.g. 1 for YES, 2 for NO in binary markets)
        #[arg(long)]
        index_set: u64,
        /// Parent collection ID (defaults to zero for top-level positions)
        #[arg(long)]
        parent_collection: Option<B256>,
    },
    /// Calculate a position ID (ERC1155 token ID) from collateral and collection
    PositionId {
        /// Collateral token address (defaults to USDC)
        #[arg(long, default_value = USDC_ADDRESS_STR)]
        collateral: Address,
        /// Collection ID (0x-prefixed 32-byte hex)
        #[arg(long)]
        collection: B256,
    },
}

fn usdc_to_raw(val: Decimal) -> Result<U256> {
    let raw = val * USDC_DECIMALS;
    anyhow::ensure!(
        raw.fract().is_zero(),
        "Amount {val} exceeds USDC precision (max 6 decimal places)"
    );
    let raw_u64: u64 = raw
        .try_into()
        .map_err(|_| anyhow::anyhow!("Amount too large: {val}"))?;
    Ok(U256::from(raw_u64))
}

fn parse_usdc_amount(s: &str) -> Result<U256> {
    let val: Decimal = s.trim().parse().context(format!("Invalid amount: {s}"))?;
    anyhow::ensure!(val > Decimal::ZERO, "Amount must be positive");
    usdc_to_raw(val)
}

fn parse_usdc_amounts(s: &str) -> Result<Vec<U256>> {
    s.split(',')
        .map(|part| {
            let trimmed = part.trim();
            let val: Decimal = trimmed
                .parse()
                .context(format!("Invalid amount: {trimmed}"))?;
            anyhow::ensure!(
                val >= Decimal::ZERO,
                "Amount must be non-negative: {trimmed}"
            );
            usdc_to_raw(val)
        })
        .collect()
}

fn parse_u256_csv(s: &str) -> Result<Vec<U256>> {
    s.split(',')
        .map(|part| {
            let trimmed = part.trim();
            let val: u64 = trimmed
                .parse()
                .context(format!("Invalid value: {trimmed}"))?;
            Ok(U256::from(val))
        })
        .collect()
}

const DEFAULT_BINARY_SETS: [u64; 2] = [1, 2];

fn binary_u256_vec() -> Vec<U256> {
    DEFAULT_BINARY_SETS.iter().map(|&n| U256::from(n)).collect()
}

pub async fn execute(args: CtfArgs, output: OutputFormat, private_key: Option<&str>) -> Result<()> {
    match args.command {
        CtfCommand::Split {
            condition,
            amount,
            collateral,
            partition,
            parent_collection,
        } => {
            let usdc_amount = parse_usdc_amount(&amount)?;
            let parent = parent_collection.unwrap_or_default();
            let partition = match partition {
                Some(p) => parse_u256_csv(&p)?,
                None => binary_u256_vec(),
            };

            let provider = auth::create_provider(private_key).await?;
            let client = ctf::Client::new(provider, POLYGON)?;

            let req = SplitPositionRequest::builder()
                .collateral_token(collateral)
                .parent_collection_id(parent)
                .condition_id(condition)
                .partition(partition)
                .amount(usdc_amount)
                .build();

            let resp = client
                .split_position(&req)
                .await
                .context("Split position failed")?;

            ctf_output::print_tx_result("split", resp.transaction_hash, resp.block_number, &output)
        }
        CtfCommand::Merge {
            condition,
            amount,
            collateral,
            partition,
            parent_collection,
        } => {
            let usdc_amount = parse_usdc_amount(&amount)?;
            let parent = parent_collection.unwrap_or_default();
            let partition = match partition {
                Some(p) => parse_u256_csv(&p)?,
                None => binary_u256_vec(),
            };

            let provider = auth::create_provider(private_key).await?;
            let client = ctf::Client::new(provider, POLYGON)?;

            let req = MergePositionsRequest::builder()
                .collateral_token(collateral)
                .parent_collection_id(parent)
                .condition_id(condition)
                .partition(partition)
                .amount(usdc_amount)
                .build();

            let resp = client
                .merge_positions(&req)
                .await
                .context("Merge positions failed")?;

            ctf_output::print_tx_result("merge", resp.transaction_hash, resp.block_number, &output)
        }
        CtfCommand::Redeem {
            condition,
            collateral,
            index_sets,
            parent_collection,
        } => {
            let parent = parent_collection.unwrap_or_default();
            let index_sets = match index_sets {
                Some(s) => parse_u256_csv(&s)?,
                None => binary_u256_vec(),
            };

            let provider = auth::create_provider(private_key).await?;
            let client = ctf::Client::new(provider, POLYGON)?;

            let req = RedeemPositionsRequest::builder()
                .collateral_token(collateral)
                .parent_collection_id(parent)
                .condition_id(condition)
                .index_sets(index_sets)
                .build();

            let resp = client
                .redeem_positions(&req)
                .await
                .context("Redeem positions failed")?;

            ctf_output::print_tx_result("redeem", resp.transaction_hash, resp.block_number, &output)
        }
        CtfCommand::RedeemNegRisk { condition, amounts } => {
            let amounts = parse_usdc_amounts(&amounts)?;

            let provider = auth::create_provider(private_key).await?;
            let client = ctf::Client::with_neg_risk(provider, POLYGON)?;

            let req = RedeemNegRiskRequest::builder()
                .condition_id(condition)
                .amounts(amounts)
                .build();

            let resp = client
                .redeem_neg_risk(&req)
                .await
                .context("Redeem neg-risk positions failed")?;

            ctf_output::print_tx_result(
                "redeem-neg-risk",
                resp.transaction_hash,
                resp.block_number,
                &output,
            )
        }
        CtfCommand::ConditionId {
            oracle,
            question,
            outcomes,
        } => {
            let provider = auth::create_readonly_provider().await?;
            let client = ctf::Client::new(provider, POLYGON)?;

            let req = ConditionIdRequest::builder()
                .oracle(oracle)
                .question_id(question)
                .outcome_slot_count(U256::from(outcomes))
                .build();

            let resp = client.condition_id(&req).await?;
            ctf_output::print_condition_id(resp.condition_id, &output)
        }
        CtfCommand::CollectionId {
            condition,
            index_set,
            parent_collection,
        } => {
            let parent = parent_collection.unwrap_or_default();

            let provider = auth::create_readonly_provider().await?;
            let client = ctf::Client::new(provider, POLYGON)?;

            let req = CollectionIdRequest::builder()
                .parent_collection_id(parent)
                .condition_id(condition)
                .index_set(U256::from(index_set))
                .build();

            let resp = client.collection_id(&req).await?;
            ctf_output::print_collection_id(resp.collection_id, &output)
        }
        CtfCommand::PositionId {
            collateral,
            collection,
        } => {
            let provider = auth::create_readonly_provider().await?;
            let client = ctf::Client::new(provider, POLYGON)?;

            let req = PositionIdRequest::builder()
                .collateral_token(collateral)
                .collection_id(collection)
                .build();

            let resp = client.position_id(&req).await?;
            ctf_output::print_position_id(resp.position_id, &output)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_usdc_amount_whole_dollars() {
        let result = parse_usdc_amount("10").unwrap();
        assert_eq!(result, U256::from(10_000_000u64));
    }

    #[test]
    fn parse_usdc_amount_fractional() {
        let result = parse_usdc_amount("1.5").unwrap();
        assert_eq!(result, U256::from(1_500_000u64));
    }

    #[test]
    fn parse_usdc_amount_small() {
        let result = parse_usdc_amount("0.01").unwrap();
        assert_eq!(result, U256::from(10_000u64));
    }

    #[test]
    fn parse_usdc_amount_smallest_unit() {
        let result = parse_usdc_amount("0.000001").unwrap();
        assert_eq!(result, U256::from(1u64));
    }

    #[test]
    fn parse_usdc_amount_rejects_excess_precision() {
        let err = parse_usdc_amount("1.0000001").unwrap_err().to_string();
        assert!(err.contains("precision"), "got: {err}");
    }

    #[test]
    fn parse_usdc_amount_rejects_zero() {
        assert!(parse_usdc_amount("0").is_err());
    }

    #[test]
    fn parse_usdc_amount_rejects_negative() {
        assert!(parse_usdc_amount("-5").is_err());
    }

    #[test]
    fn parse_usdc_amount_rejects_non_numeric() {
        assert!(parse_usdc_amount("abc").is_err());
    }

    #[test]
    fn parse_usdc_amounts_single() {
        let result = parse_usdc_amounts("10").unwrap();
        assert_eq!(result, vec![U256::from(10_000_000u64)]);
    }

    #[test]
    fn parse_usdc_amounts_multiple() {
        let result = parse_usdc_amounts("10,5").unwrap();
        assert_eq!(
            result,
            vec![U256::from(10_000_000u64), U256::from(5_000_000u64)]
        );
    }

    #[test]
    fn parse_usdc_amounts_with_spaces() {
        let result = parse_usdc_amounts("10, 5, 2.5").unwrap();
        assert_eq!(
            result,
            vec![
                U256::from(10_000_000u64),
                U256::from(5_000_000u64),
                U256::from(2_500_000u64)
            ]
        );
    }

    #[test]
    fn parse_usdc_amounts_zero_is_allowed() {
        let result = parse_usdc_amounts("0,10").unwrap();
        assert_eq!(result, vec![U256::from(0u64), U256::from(10_000_000u64)]);
    }

    #[test]
    fn parse_usdc_amounts_rejects_negative() {
        assert!(parse_usdc_amounts("10,-5").is_err());
    }

    #[test]
    fn parse_usdc_amounts_rejects_non_numeric() {
        assert!(parse_usdc_amounts("abc").is_err());
    }

    #[test]
    fn parse_u256_csv_binary_partition() {
        let result = parse_u256_csv("1,2").unwrap();
        assert_eq!(result, vec![U256::from(1u64), U256::from(2u64)]);
    }

    #[test]
    fn parse_u256_csv_three_outcome() {
        let result = parse_u256_csv("1,2,4").unwrap();
        assert_eq!(
            result,
            vec![U256::from(1u64), U256::from(2u64), U256::from(4u64)]
        );
    }

    #[test]
    fn parse_u256_csv_with_spaces() {
        let result = parse_u256_csv("1, 2, 4").unwrap();
        assert_eq!(
            result,
            vec![U256::from(1u64), U256::from(2u64), U256::from(4u64)]
        );
    }

    #[test]
    fn parse_u256_csv_single() {
        let result = parse_u256_csv("1").unwrap();
        assert_eq!(result, vec![U256::from(1u64)]);
    }

    #[test]
    fn parse_u256_csv_rejects_non_numeric() {
        assert!(parse_u256_csv("abc").is_err());
    }

    #[test]
    fn parse_u256_csv_rejects_partial_invalid() {
        assert!(parse_u256_csv("1,abc,3").is_err());
    }

    #[test]
    fn binary_u256_vec_is_binary() {
        let p = binary_u256_vec();
        assert_eq!(p, vec![U256::from(1u64), U256::from(2u64)]);
    }
}
