use anyhow::{Context, Result};
use polymarket_client_sdk_v2::{data, gamma};

fn host(variable: &str, default: &str) -> String {
    normalize_host(&std::env::var(variable).unwrap_or_else(|_| default.to_owned()))
}

fn normalize_host(host: &str) -> String {
    // SDK requests append paths to the base URL; preserve prefixes with one slash.
    format!("{}/", host.trim_end_matches('/'))
}

pub fn gamma_client() -> Result<gamma::Client> {
    gamma::Client::new(&host(
        "POLYMARKET_GAMMA_HOST",
        "https://gamma-api.polymarket.com",
    ))
    .context("Failed to create Gamma client; check POLYMARKET_GAMMA_HOST")
}

pub fn data_client() -> Result<data::Client> {
    data::Client::new(&host(
        "POLYMARKET_DATA_HOST",
        "https://data-api.polymarket.com",
    ))
    .context("Failed to create Data client; check POLYMARKET_DATA_HOST")
}

pub fn clob_host() -> String {
    host("POLYMARKET_CLOB_HOST", "https://clob.polymarket.com")
}

#[cfg(test)]
mod tests {
    use super::normalize_host;

    #[test]
    fn preserves_service_prefix_with_one_trailing_slash() {
        for host in [
            "http://127.0.0.1:4400/gamma",
            "http://127.0.0.1:4400/gamma/",
            "http://127.0.0.1:4400/gamma///",
        ] {
            assert_eq!(normalize_host(host), "http://127.0.0.1:4400/gamma/");
        }
        assert_eq!(
            normalize_host("https://clob.polymarket.com"),
            "https://clob.polymarket.com/"
        );
    }
}
