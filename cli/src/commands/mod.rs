use polymarket_client_sdk::types::{Address, B256};

pub mod bridge;
pub mod clob;
pub mod comments;
pub mod data;
pub mod events;
pub mod markets;
pub mod profiles;
pub mod series;
pub mod sports;
pub mod tags;
pub mod wallet;

pub fn is_numeric_id(id: &str) -> bool {
    !id.is_empty() && id.chars().all(|c| c.is_ascii_digit())
}

pub fn parse_address(s: &str) -> anyhow::Result<Address> {
    s.parse()
        .map_err(|_| anyhow::anyhow!("Invalid address: must be a 0x-prefixed hex address"))
}

pub fn parse_condition_id(s: &str) -> anyhow::Result<B256> {
    s.parse()
        .map_err(|_| anyhow::anyhow!("Invalid condition ID: must be a 0x-prefixed 32-byte hex"))
}
