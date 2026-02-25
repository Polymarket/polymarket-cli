use polymarket_client_sdk::types::{Address, B256};

pub mod approve;
pub mod bridge;
pub mod clob;
pub mod comments;
pub mod ctf;
pub mod data;
pub mod events;
pub mod markets;
pub mod metengine;
pub mod profiles;
pub mod series;
pub mod setup;
pub mod sports;
pub mod tags;
pub mod upgrade;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_numeric_id_pure_digits() {
        assert!(is_numeric_id("12345"));
        assert!(is_numeric_id("0"));
    }

    #[test]
    fn is_numeric_id_rejects_non_digits() {
        assert!(!is_numeric_id("will-trump-win"));
        assert!(!is_numeric_id("0x123abc"));
        assert!(!is_numeric_id("123 456"));
    }

    #[test]
    fn is_numeric_id_rejects_empty() {
        assert!(!is_numeric_id(""));
    }

    #[test]
    fn parse_address_valid_hex() {
        let addr = "0x0000000000000000000000000000000000000001";
        assert!(parse_address(addr).is_ok());
    }

    #[test]
    fn parse_address_rejects_short_hex() {
        let err = parse_address("0x1234").unwrap_err().to_string();
        assert!(err.contains("0x-prefixed"), "got: {err}");
    }

    #[test]
    fn parse_address_rejects_garbage() {
        let err = parse_address("not-an-address").unwrap_err().to_string();
        assert!(err.contains("0x-prefixed"), "got: {err}");
    }

    #[test]
    fn parse_condition_id_valid_64_hex() {
        let id = "0x0000000000000000000000000000000000000000000000000000000000000001";
        assert!(parse_condition_id(id).is_ok());
    }

    #[test]
    fn parse_condition_id_rejects_wrong_length() {
        let err = parse_condition_id("0x0001").unwrap_err().to_string();
        assert!(err.contains("32-byte"), "got: {err}");
    }

    #[test]
    fn parse_condition_id_rejects_garbage() {
        let err = parse_condition_id("garbage").unwrap_err().to_string();
        assert!(err.contains("32-byte"), "got: {err}");
    }
}
