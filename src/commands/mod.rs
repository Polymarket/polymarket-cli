/// Polygon USDC contract address (shared across ctf and approve commands).
pub const USDC_ADDRESS_STR: &str = "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174";

pub mod approve;
pub mod bridge;
pub mod clob;
pub mod comments;
pub mod ctf;
pub mod data;
pub mod events;
pub mod markets;
pub mod profiles;
pub mod series;
pub mod setup;
pub mod sports;
pub mod tags;
pub mod upgrade;
pub mod wallet;

pub fn is_numeric_id(id: &str) -> bool {
    id.parse::<u64>().is_ok()
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
}
