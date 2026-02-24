use alloy::primitives::{Bytes, U256};
use alloy::sol;
use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use polymarket_client_sdk::ctf::types::{
    CollectionIdRequest, ConditionIdRequest, MergePositionsRequest, PositionIdRequest,
    RedeemNegRiskRequest, RedeemPositionsRequest, SplitPositionRequest,
};
use polymarket_client_sdk::types::{Address, B256};
use polymarket_client_sdk::{POLYGON, contract_config, ctf};
use rust_decimal::Decimal;

use crate::auth;
use crate::output::OutputFormat;
use crate::output::ctf as ctf_output;

sol! {
    #[allow(clippy::exhaustive_enums)]
    #[allow(clippy::exhaustive_structs)]
    interface IConditionalTokens {
        function splitPosition(
            address collateralToken,
            bytes32 parentCollectionId,
            bytes32 conditionId,
            uint256[] calldata partition,
            uint256 amount
        ) external;

        function mergePositions(
            address collateralToken,
            bytes32 parentCollectionId,
            bytes32 conditionId,
            uint256[] calldata partition,
            uint256 amount
        ) external;

        function redeemPositions(
            address collateralToken,
            bytes32 parentCollectionId,
            bytes32 conditionId,
            uint256[] calldata indexSets
        ) external;
    }

    #[allow(clippy::exhaustive_enums)]
    #[allow(clippy::exhaustive_structs)]
    interface INegRiskAdapter {
        function redeemPositions(
            bytes32 conditionId,
            uint256[] calldata amounts
        ) external;
    }
}

const USDC_DECIMALS: Decimal = Decimal::from_parts(1_000_000, 0, 0, false, 0);

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
        condition: String,
        /// Amount in USDC (e.g. 10 for $10)
        #[arg(long)]
        amount: String,
        /// Collateral token address (defaults to USDC)
        #[arg(long, default_value = "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174")]
        collateral: String,
        /// Custom partition as comma-separated index sets (e.g. "1,2" for binary, "1,2,4" for 3-outcome)
        #[arg(long)]
        partition: Option<String>,
        /// Parent collection ID for nested positions (defaults to zero)
        #[arg(long)]
        parent_collection: Option<String>,
    },
    /// Merge outcome tokens back into collateral
    Merge {
        /// Condition ID (0x-prefixed 32-byte hex)
        #[arg(long)]
        condition: String,
        /// Amount in USDC (e.g. 10 for $10)
        #[arg(long)]
        amount: String,
        /// Collateral token address (defaults to USDC)
        #[arg(long, default_value = "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174")]
        collateral: String,
        /// Custom partition as comma-separated index sets (e.g. "1,2" for binary, "1,2,4" for 3-outcome)
        #[arg(long)]
        partition: Option<String>,
        /// Parent collection ID for nested positions (defaults to zero)
        #[arg(long)]
        parent_collection: Option<String>,
    },
    /// Redeem winning tokens after market resolution
    Redeem {
        /// Condition ID (0x-prefixed 32-byte hex)
        #[arg(long)]
        condition: String,
        /// Collateral token address (defaults to USDC)
        #[arg(long, default_value = "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174")]
        collateral: String,
        /// Custom index sets as comma-separated values (e.g. "1,2" for binary, "1" for YES only)
        #[arg(long)]
        index_sets: Option<String>,
        /// Parent collection ID for nested positions (defaults to zero)
        #[arg(long)]
        parent_collection: Option<String>,
    },
    /// Redeem neg-risk positions
    RedeemNegRisk {
        /// Condition ID (0x-prefixed 32-byte hex)
        #[arg(long)]
        condition: String,
        /// Comma-separated amounts in USDC for each outcome (e.g. "10,5")
        #[arg(long)]
        amounts: String,
    },
    /// Calculate a condition ID from oracle, question, and outcome count
    ConditionId {
        /// Oracle address (0x-prefixed)
        #[arg(long)]
        oracle: String,
        /// Question ID (0x-prefixed 32-byte hex)
        #[arg(long)]
        question: String,
        /// Number of outcomes (e.g. 2 for binary)
        #[arg(long)]
        outcomes: u64,
    },
    /// Calculate a collection ID from condition and index set
    CollectionId {
        /// Condition ID (0x-prefixed 32-byte hex)
        #[arg(long)]
        condition: String,
        /// Index set (e.g. 1 for YES, 2 for NO in binary markets)
        #[arg(long)]
        index_set: u64,
        /// Parent collection ID (defaults to zero for top-level positions)
        #[arg(long)]
        parent_collection: Option<String>,
    },
    /// Calculate a position ID (ERC1155 token ID) from collateral and collection
    PositionId {
        /// Collateral token address (defaults to USDC)
        #[arg(long, default_value = "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174")]
        collateral: String,
        /// Collection ID (0x-prefixed 32-byte hex)
        #[arg(long)]
        collection: String,
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

fn parse_optional_parent(parent: Option<&str>) -> Result<B256> {
    match parent {
        Some(p) => super::parse_condition_id(p),
        None => Ok(B256::default()),
    }
}

fn resolve_collateral(collateral: &str) -> Result<Address> {
    super::parse_address(collateral)
}

fn default_partition() -> Vec<U256> {
    vec![U256::from(1), U256::from(2)]
}

fn default_index_sets() -> Vec<U256> {
    vec![U256::from(1), U256::from(2)]
}

pub async fn execute(
    args: CtfArgs,
    output: OutputFormat,
    private_key: Option<&str>,
    signature_type: Option<&str>,
) -> Result<()> {
    match args.command {
        CtfCommand::Split {
            condition,
            amount,
            collateral,
            partition,
            parent_collection,
        } => {
            let condition_id = super::parse_condition_id(&condition)?;
            let usdc_amount = parse_usdc_amount(&amount)?;
            let collateral_addr = resolve_collateral(&collateral)?;
            let parent = parse_optional_parent(parent_collection.as_deref())?;
            let partition = match partition {
                Some(p) => parse_u256_csv(&p)?,
                None => default_partition(),
            };

            let proxy_addr = auth::resolve_proxy_address(private_key, signature_type)?;
            let provider = auth::create_provider(private_key).await?;

            if let Some(proxy) = proxy_addr {
                let config =
                    contract_config(POLYGON, false).context("No contract config for Polygon")?;
                let calldata = IConditionalTokens::splitPositionCall {
                    collateralToken: collateral_addr,
                    parentCollectionId: parent,
                    conditionId: condition_id,
                    partition,
                    amount: usdc_amount,
                };
                let tx_hash = auth::proxy_exec(
                    &provider,
                    proxy,
                    config.conditional_tokens,
                    Bytes::from(alloy::sol_types::SolCall::abi_encode(&calldata)),
                )
                .await
                .context("Split position via proxy failed")?;
                ctf_output::print_tx_result("split", tx_hash, None, &output)
            } else {
                let client = ctf::Client::new(provider, POLYGON)?;
                let req = SplitPositionRequest::builder()
                    .collateral_token(collateral_addr)
                    .parent_collection_id(parent)
                    .condition_id(condition_id)
                    .partition(partition)
                    .amount(usdc_amount)
                    .build();
                let resp = client
                    .split_position(&req)
                    .await
                    .context("Split position failed")?;
                ctf_output::print_tx_result(
                    "split",
                    resp.transaction_hash,
                    Some(resp.block_number),
                    &output,
                )
            }
        }
        CtfCommand::Merge {
            condition,
            amount,
            collateral,
            partition,
            parent_collection,
        } => {
            let condition_id = super::parse_condition_id(&condition)?;
            let usdc_amount = parse_usdc_amount(&amount)?;
            let collateral_addr = resolve_collateral(&collateral)?;
            let parent = parse_optional_parent(parent_collection.as_deref())?;
            let partition = match partition {
                Some(p) => parse_u256_csv(&p)?,
                None => default_partition(),
            };

            let proxy_addr = auth::resolve_proxy_address(private_key, signature_type)?;
            let provider = auth::create_provider(private_key).await?;

            if let Some(proxy) = proxy_addr {
                let config =
                    contract_config(POLYGON, false).context("No contract config for Polygon")?;
                let calldata = IConditionalTokens::mergePositionsCall {
                    collateralToken: collateral_addr,
                    parentCollectionId: parent,
                    conditionId: condition_id,
                    partition,
                    amount: usdc_amount,
                };
                let tx_hash = auth::proxy_exec(
                    &provider,
                    proxy,
                    config.conditional_tokens,
                    Bytes::from(alloy::sol_types::SolCall::abi_encode(&calldata)),
                )
                .await
                .context("Merge positions via proxy failed")?;
                ctf_output::print_tx_result("merge", tx_hash, None, &output)
            } else {
                let client = ctf::Client::new(provider, POLYGON)?;
                let req = MergePositionsRequest::builder()
                    .collateral_token(collateral_addr)
                    .parent_collection_id(parent)
                    .condition_id(condition_id)
                    .partition(partition)
                    .amount(usdc_amount)
                    .build();
                let resp = client
                    .merge_positions(&req)
                    .await
                    .context("Merge positions failed")?;
                ctf_output::print_tx_result(
                    "merge",
                    resp.transaction_hash,
                    Some(resp.block_number),
                    &output,
                )
            }
        }
        CtfCommand::Redeem {
            condition,
            collateral,
            index_sets,
            parent_collection,
        } => {
            let condition_id = super::parse_condition_id(&condition)?;
            let collateral_addr = resolve_collateral(&collateral)?;
            let parent = parse_optional_parent(parent_collection.as_deref())?;
            let index_sets = match index_sets {
                Some(s) => parse_u256_csv(&s)?,
                None => default_index_sets(),
            };

            let proxy_addr = auth::resolve_proxy_address(private_key, signature_type)?;
            let provider = auth::create_provider(private_key).await?;

            if let Some(proxy) = proxy_addr {
                let config =
                    contract_config(POLYGON, false).context("No contract config for Polygon")?;
                let calldata = IConditionalTokens::redeemPositionsCall {
                    collateralToken: collateral_addr,
                    parentCollectionId: parent,
                    conditionId: condition_id,
                    indexSets: index_sets,
                };
                let tx_hash = auth::proxy_exec(
                    &provider,
                    proxy,
                    config.conditional_tokens,
                    Bytes::from(alloy::sol_types::SolCall::abi_encode(&calldata)),
                )
                .await
                .context("Redeem positions via proxy failed")?;
                ctf_output::print_tx_result("redeem", tx_hash, None, &output)
            } else {
                let client = ctf::Client::new(provider, POLYGON)?;
                let req = RedeemPositionsRequest::builder()
                    .collateral_token(collateral_addr)
                    .parent_collection_id(parent)
                    .condition_id(condition_id)
                    .index_sets(index_sets)
                    .build();
                let resp = client
                    .redeem_positions(&req)
                    .await
                    .context("Redeem positions failed")?;
                ctf_output::print_tx_result(
                    "redeem",
                    resp.transaction_hash,
                    Some(resp.block_number),
                    &output,
                )
            }
        }
        CtfCommand::RedeemNegRisk { condition, amounts } => {
            let condition_id = super::parse_condition_id(&condition)?;
            let amounts = parse_usdc_amounts(&amounts)?;

            let proxy_addr = auth::resolve_proxy_address(private_key, signature_type)?;
            let provider = auth::create_provider(private_key).await?;

            if let Some(proxy) = proxy_addr {
                let neg_risk_config = contract_config(POLYGON, true)
                    .context("No neg-risk contract config for Polygon")?;
                let adapter = neg_risk_config
                    .neg_risk_adapter
                    .context("No neg-risk adapter address configured")?;
                let calldata = INegRiskAdapter::redeemPositionsCall {
                    conditionId: condition_id,
                    amounts,
                };
                let tx_hash = auth::proxy_exec(
                    &provider,
                    proxy,
                    adapter,
                    Bytes::from(alloy::sol_types::SolCall::abi_encode(&calldata)),
                )
                .await
                .context("Redeem neg-risk positions via proxy failed")?;
                ctf_output::print_tx_result("redeem-neg-risk", tx_hash, None, &output)
            } else {
                let client = ctf::Client::with_neg_risk(provider, POLYGON)?;
                let req = RedeemNegRiskRequest::builder()
                    .condition_id(condition_id)
                    .amounts(amounts)
                    .build();
                let resp = client
                    .redeem_neg_risk(&req)
                    .await
                    .context("Redeem neg-risk positions failed")?;
                ctf_output::print_tx_result(
                    "redeem-neg-risk",
                    resp.transaction_hash,
                    Some(resp.block_number),
                    &output,
                )
            }
        }
        CtfCommand::ConditionId {
            oracle,
            question,
            outcomes,
        } => {
            let oracle_addr = super::parse_address(&oracle)?;
            let question_id = super::parse_condition_id(&question)?;

            let provider = auth::create_readonly_provider().await?;
            let client = ctf::Client::new(provider, POLYGON)?;

            let req = ConditionIdRequest::builder()
                .oracle(oracle_addr)
                .question_id(question_id)
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
            let condition_id = super::parse_condition_id(&condition)?;
            let parent = parse_optional_parent(parent_collection.as_deref())?;

            let provider = auth::create_readonly_provider().await?;
            let client = ctf::Client::new(provider, POLYGON)?;

            let req = CollectionIdRequest::builder()
                .parent_collection_id(parent)
                .condition_id(condition_id)
                .index_set(U256::from(index_set))
                .build();

            let resp = client.collection_id(&req).await?;
            ctf_output::print_collection_id(resp.collection_id, &output)
        }
        CtfCommand::PositionId {
            collateral,
            collection,
        } => {
            let collateral_addr = super::parse_address(&collateral)?;
            let collection_id = super::parse_condition_id(&collection)?;

            let provider = auth::create_readonly_provider().await?;
            let client = ctf::Client::new(provider, POLYGON)?;

            let req = PositionIdRequest::builder()
                .collateral_token(collateral_addr)
                .collection_id(collection_id)
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
    fn parse_optional_parent_none_is_zero() {
        let result = parse_optional_parent(None).unwrap();
        assert_eq!(result, B256::default());
    }

    #[test]
    fn parse_optional_parent_some_parses() {
        let hex = "0x0000000000000000000000000000000000000000000000000000000000000001";
        let result = parse_optional_parent(Some(hex)).unwrap();
        assert_ne!(result, B256::default());
    }

    #[test]
    fn parse_optional_parent_invalid_fails() {
        assert!(parse_optional_parent(Some("garbage")).is_err());
    }

    #[test]
    fn default_partition_is_binary() {
        let p = default_partition();
        assert_eq!(p, vec![U256::from(1u64), U256::from(2u64)]);
    }

    #[test]
    fn default_index_sets_is_binary() {
        let s = default_index_sets();
        assert_eq!(s, vec![U256::from(1u64), U256::from(2u64)]);
    }
}
