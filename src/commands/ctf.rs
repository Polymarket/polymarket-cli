use alloy::primitives::U256;
use alloy::sol;
use alloy::sol_types::SolCall;
use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use polymarket_client_sdk_v2::types::{Address, B256};
use rust_decimal::Decimal;

use crate::auth;
use crate::output::OutputFormat;
use crate::output::ctf as ctf_output;

use super::proxy;
use super::{
    COLLATERAL_ADDRESS_STR, COLLATERAL_DECIMALS, COLLATERAL_SYMBOL, CONDITIONAL_TOKENS,
    NEG_RISK_ADAPTER,
};

sol! {
    #[sol(rpc)]
    interface IConditionalTokens {
        function getConditionId(
            address oracle,
            bytes32 questionId,
            uint256 outcomeSlotCount
        ) external pure returns (bytes32);

        function getCollectionId(
            bytes32 parentCollectionId,
            bytes32 conditionId,
            uint256 indexSet
        ) external view returns (bytes32);

        function getPositionId(
            address collateralToken,
            bytes32 collectionId
        ) external view returns (uint256);

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

    interface INegRiskAdapter {
        function redeemPositions(
            bytes32 conditionId,
            uint256[] calldata amounts
        ) external;
    }
}

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
        /// Amount in pUSD (e.g. 10 for $10)
        #[arg(long)]
        amount: String,
        /// Collateral token address (defaults to pUSD)
        #[arg(long, default_value = COLLATERAL_ADDRESS_STR)]
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
        /// Amount in pUSD (e.g. 10 for $10)
        #[arg(long)]
        amount: String,
        /// Collateral token address (defaults to pUSD)
        #[arg(long, default_value = COLLATERAL_ADDRESS_STR)]
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
        /// Collateral token address (defaults to pUSD)
        #[arg(long, default_value = COLLATERAL_ADDRESS_STR)]
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
        /// Comma-separated amounts in pUSD for each outcome (e.g. "10,5")
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
        /// Collateral token address (defaults to pUSD)
        #[arg(long, default_value = COLLATERAL_ADDRESS_STR)]
        collateral: Address,
        /// Collection ID (0x-prefixed 32-byte hex)
        #[arg(long)]
        collection: B256,
    },
}

fn collateral_to_raw(val: Decimal) -> Result<U256> {
    let multiplier = Decimal::from(10u64.pow(COLLATERAL_DECIMALS));
    let raw = val * multiplier;
    anyhow::ensure!(
        raw.fract().is_zero(),
        "Amount {val} exceeds {COLLATERAL_SYMBOL} precision (max {COLLATERAL_DECIMALS} decimal places)"
    );
    let raw_u64: u64 = raw
        .try_into()
        .map_err(|_| anyhow::anyhow!("Amount too large: {val}"))?;
    Ok(U256::from(raw_u64))
}

fn parse_collateral_amount(s: &str) -> Result<U256> {
    let val: Decimal = s.trim().parse().context(format!("Invalid amount: {s}"))?;
    anyhow::ensure!(val > Decimal::ZERO, "Amount must be positive");
    collateral_to_raw(val)
}

fn parse_collateral_amounts(s: &str) -> Result<Vec<U256>> {
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
            collateral_to_raw(val)
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
            let collateral_amount = parse_collateral_amount(&amount)?;
            let parent = parent_collection.unwrap_or_default();
            let partition = match partition {
                Some(p) => parse_u256_csv(&p)?,
                None => binary_u256_vec(),
            };

            let use_proxy = proxy::is_proxy_mode(signature_type)?;
            let calldata = IConditionalTokens::splitPositionCall {
                collateralToken: collateral,
                parentCollectionId: parent,
                conditionId: condition,
                partition,
                amount: collateral_amount,
            }
            .abi_encode();

            let (tx_hash, block_number) =
                proxy::send_call(private_key, use_proxy, CONDITIONAL_TOKENS, calldata)
                    .await
                    .context("Split position failed")?;

            ctf_output::print_tx_result("split", tx_hash, block_number, &output)
        }
        CtfCommand::Merge {
            condition,
            amount,
            collateral,
            partition,
            parent_collection,
        } => {
            let collateral_amount = parse_collateral_amount(&amount)?;
            let parent = parent_collection.unwrap_or_default();
            let partition = match partition {
                Some(p) => parse_u256_csv(&p)?,
                None => binary_u256_vec(),
            };

            let use_proxy = proxy::is_proxy_mode(signature_type)?;
            let calldata = IConditionalTokens::mergePositionsCall {
                collateralToken: collateral,
                parentCollectionId: parent,
                conditionId: condition,
                partition,
                amount: collateral_amount,
            }
            .abi_encode();

            let (tx_hash, block_number) =
                proxy::send_call(private_key, use_proxy, CONDITIONAL_TOKENS, calldata)
                    .await
                    .context("Merge positions failed")?;

            ctf_output::print_tx_result("merge", tx_hash, block_number, &output)
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

            let use_proxy = proxy::is_proxy_mode(signature_type)?;
            let calldata = IConditionalTokens::redeemPositionsCall {
                collateralToken: collateral,
                parentCollectionId: parent,
                conditionId: condition,
                indexSets: index_sets,
            }
            .abi_encode();

            let (tx_hash, block_number) =
                proxy::send_call(private_key, use_proxy, CONDITIONAL_TOKENS, calldata)
                    .await
                    .context("Redeem positions failed")?;

            ctf_output::print_tx_result("redeem", tx_hash, block_number, &output)
        }
        CtfCommand::RedeemNegRisk { condition, amounts } => {
            let amounts = parse_collateral_amounts(&amounts)?;

            let use_proxy = proxy::is_proxy_mode(signature_type)?;
            let calldata = INegRiskAdapter::redeemPositionsCall {
                conditionId: condition,
                amounts,
            }
            .abi_encode();

            let (tx_hash, block_number) =
                proxy::send_call(private_key, use_proxy, NEG_RISK_ADAPTER, calldata)
                    .await
                    .context("Redeem neg-risk positions failed")?;

            ctf_output::print_tx_result("redeem-neg-risk", tx_hash, block_number, &output)
        }
        CtfCommand::ConditionId {
            oracle,
            question,
            outcomes,
        } => {
            let provider = auth::create_readonly_provider().await?;
            let contract = IConditionalTokens::new(CONDITIONAL_TOKENS, provider);
            let condition_id = contract
                .getConditionId(oracle, question, U256::from(outcomes))
                .call()
                .await?;
            ctf_output::print_condition_id(condition_id, &output)
        }
        CtfCommand::CollectionId {
            condition,
            index_set,
            parent_collection,
        } => {
            let parent = parent_collection.unwrap_or_default();

            let provider = auth::create_readonly_provider().await?;
            let contract = IConditionalTokens::new(CONDITIONAL_TOKENS, provider);
            let collection_id = contract
                .getCollectionId(parent, condition, U256::from(index_set))
                .call()
                .await?;
            ctf_output::print_collection_id(collection_id, &output)
        }
        CtfCommand::PositionId {
            collateral,
            collection,
        } => {
            let provider = auth::create_readonly_provider().await?;
            let contract = IConditionalTokens::new(CONDITIONAL_TOKENS, provider);
            let position_id = contract
                .getPositionId(collateral, collection)
                .call()
                .await?;
            ctf_output::print_position_id(position_id, &output)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_collateral_amount_whole_dollars() {
        let result = parse_collateral_amount("10").unwrap();
        assert_eq!(result, U256::from(10_000_000u64));
    }

    #[test]
    fn parse_collateral_amount_fractional() {
        let result = parse_collateral_amount("1.5").unwrap();
        assert_eq!(result, U256::from(1_500_000u64));
    }

    #[test]
    fn parse_collateral_amount_small() {
        let result = parse_collateral_amount("0.01").unwrap();
        assert_eq!(result, U256::from(10_000u64));
    }

    #[test]
    fn parse_collateral_amount_smallest_unit() {
        let result = parse_collateral_amount("0.000001").unwrap();
        assert_eq!(result, U256::from(1u64));
    }

    #[test]
    fn parse_collateral_amount_rejects_excess_precision() {
        let err = parse_collateral_amount("1.0000001")
            .unwrap_err()
            .to_string();
        assert!(err.contains("precision"), "got: {err}");
    }

    #[test]
    fn parse_collateral_amount_rejects_zero() {
        assert!(parse_collateral_amount("0").is_err());
    }

    #[test]
    fn parse_collateral_amount_rejects_negative() {
        assert!(parse_collateral_amount("-5").is_err());
    }

    #[test]
    fn parse_collateral_amount_rejects_non_numeric() {
        assert!(parse_collateral_amount("abc").is_err());
    }

    #[test]
    fn parse_collateral_amounts_single() {
        let result = parse_collateral_amounts("10").unwrap();
        assert_eq!(result, vec![U256::from(10_000_000u64)]);
    }

    #[test]
    fn parse_collateral_amounts_multiple() {
        let result = parse_collateral_amounts("10,5").unwrap();
        assert_eq!(
            result,
            vec![U256::from(10_000_000u64), U256::from(5_000_000u64)]
        );
    }

    #[test]
    fn parse_collateral_amounts_with_spaces() {
        let result = parse_collateral_amounts("10, 5, 2.5").unwrap();
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
    fn parse_collateral_amounts_zero_is_allowed() {
        let result = parse_collateral_amounts("0,10").unwrap();
        assert_eq!(result, vec![U256::from(0u64), U256::from(10_000_000u64)]);
    }

    #[test]
    fn parse_collateral_amounts_rejects_negative() {
        assert!(parse_collateral_amounts("10,-5").is_err());
    }

    #[test]
    fn parse_collateral_amounts_rejects_non_numeric() {
        assert!(parse_collateral_amounts("abc").is_err());
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
