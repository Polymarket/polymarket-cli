use alloy::primitives::{B256, U256};
use anyhow::Result;

use super::{OutputFormat, print_detail_table};

pub fn print_tx_result(
    operation: &str,
    tx_hash: B256,
    block_number: Option<u64>,
    output: &OutputFormat,
) -> Result<()> {
    match output {
        OutputFormat::Json => {
            let json = serde_json::json!({
                "operation": operation,
                "transaction_hash": format!("{tx_hash}"),
                "block_number": block_number,
                "polygonscan": format!("https://polygonscan.com/tx/{tx_hash}"),
            });
            println!("{}", serde_json::to_string_pretty(&json)?);
            Ok(())
        }
        OutputFormat::Table => {
            let mut rows = vec![
                ["Operation".into(), operation.to_string()],
                ["Tx Hash".into(), format!("{tx_hash}")],
            ];
            if let Some(block) = block_number {
                rows.push(["Block".into(), block.to_string()]);
            }
            rows.push([
                "Polygonscan".into(),
                format!("https://polygonscan.com/tx/{tx_hash}"),
            ]);
            print_detail_table(rows);
            Ok(())
        }
    }
}

pub fn print_condition_id(condition_id: B256, output: &OutputFormat) -> Result<()> {
    match output {
        OutputFormat::Json => {
            let json = serde_json::json!({
                "condition_id": format!("{condition_id}"),
            });
            println!("{}", serde_json::to_string_pretty(&json)?);
            Ok(())
        }
        OutputFormat::Table => {
            println!("Condition ID: {condition_id}");
            Ok(())
        }
    }
}

pub fn print_collection_id(collection_id: B256, output: &OutputFormat) -> Result<()> {
    match output {
        OutputFormat::Json => {
            let json = serde_json::json!({
                "collection_id": format!("{collection_id}"),
            });
            println!("{}", serde_json::to_string_pretty(&json)?);
            Ok(())
        }
        OutputFormat::Table => {
            println!("Collection ID: {collection_id}");
            Ok(())
        }
    }
}

pub fn print_position_id(position_id: U256, output: &OutputFormat) -> Result<()> {
    match output {
        OutputFormat::Json => {
            let json = serde_json::json!({
                "position_id": position_id.to_string(),
            });
            println!("{}", serde_json::to_string_pretty(&json)?);
            Ok(())
        }
        OutputFormat::Table => {
            println!("Position ID: {position_id}");
            Ok(())
        }
    }
}
