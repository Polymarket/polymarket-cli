use polymarket_client_sdk::types::{Address, B256};

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

/// Parsed Polymarket URL with event slug and optional market slug.
#[derive(Debug, PartialEq)]
pub struct PolymarketUrl {
    pub event_slug: String,
    pub market_slug: Option<String>,
}

/// Parse a Polymarket URL into its event and optional market slugs.
///
/// Accepts URLs with or without scheme (`https://`, `http://`), with or without
/// `www.`, and strips query strings, fragments, and trailing slashes.
///
/// Returns `None` for non-Polymarket URLs or URLs missing `/event/<slug>`.
pub fn parse_polymarket_url(input: &str) -> Option<PolymarketUrl> {
    // Strip scheme if present
    let without_scheme = input
        .strip_prefix("https://")
        .or_else(|| input.strip_prefix("http://"))
        .unwrap_or(input);

    // Split host from path at the first '/'
    let (host, path) = match without_scheme.find('/') {
        Some(i) => (&without_scheme[..i], &without_scheme[i..]),
        None => return None, // No path at all
    };

    // Verify it's a polymarket.com host
    let host_lower = host.to_ascii_lowercase();
    if host_lower != "polymarket.com" && host_lower != "www.polymarket.com" {
        return None;
    }

    // Strip query string and fragment
    let path = path.split('?').next().unwrap_or(path);
    let path = path.split('#').next().unwrap_or(path);

    // Strip trailing slash
    let path = path.strip_suffix('/').unwrap_or(path);

    // Expect /event/<event_slug>[/<market_slug>]
    let path = path.strip_prefix("/event/")?;
    if path.is_empty() {
        return None;
    }

    let mut segments = path.split('/');
    let event_slug = segments.next()?.to_string();
    if event_slug.is_empty() {
        return None;
    }

    let market_slug = segments
        .next()
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    Some(PolymarketUrl {
        event_slug,
        market_slug,
    })
}

/// What `resolve_id` determined the input to be.
#[derive(Debug, PartialEq)]
pub enum ResolvedId {
    /// A numeric API id (e.g. "12345").
    Numeric(String),
    /// A slug extracted from a Polymarket URL or passed directly.
    Slug(String),
}

/// Resolve a user-provided identifier that may be a Polymarket URL, a numeric
/// ID, or a plain slug.
///
/// Accepts URLs like `https://polymarket.com/event/<event>[/<market>]`.
/// When `prefer_market` is true and the URL contains a market slug, the market
/// slug is used; otherwise the event slug is used.
pub fn resolve_id(input: &str, prefer_market: bool) -> ResolvedId {
    if let Some(parsed) = parse_polymarket_url(input) {
        let slug = if prefer_market {
            parsed.market_slug.unwrap_or(parsed.event_slug)
        } else {
            parsed.event_slug
        };
        return ResolvedId::Slug(slug);
    }

    if is_numeric_id(input) {
        ResolvedId::Numeric(input.to_string())
    } else {
        ResolvedId::Slug(input.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── is_numeric_id ──────────────────────────────────────────────

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

    // ── parse_address / parse_condition_id ──────────────────────────

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

    // ── parse_polymarket_url ───────────────────────────────────────

    #[test]
    fn parse_url_standard_event() {
        let url = "https://polymarket.com/event/will-bitcoin-hit-100k";
        let parsed = parse_polymarket_url(url).unwrap();
        assert_eq!(parsed.event_slug, "will-bitcoin-hit-100k");
        assert_eq!(parsed.market_slug, None);
    }

    #[test]
    fn parse_url_event_with_market() {
        let url = "https://polymarket.com/event/will-bitcoin-hit-100k/bitcoin-100k-by-march";
        let parsed = parse_polymarket_url(url).unwrap();
        assert_eq!(parsed.event_slug, "will-bitcoin-hit-100k");
        assert_eq!(parsed.market_slug.as_deref(), Some("bitcoin-100k-by-march"));
    }

    #[test]
    fn parse_url_http_scheme() {
        let url = "http://polymarket.com/event/some-event";
        let parsed = parse_polymarket_url(url).unwrap();
        assert_eq!(parsed.event_slug, "some-event");
    }

    #[test]
    fn parse_url_no_scheme() {
        let url = "polymarket.com/event/some-event/some-market";
        let parsed = parse_polymarket_url(url).unwrap();
        assert_eq!(parsed.event_slug, "some-event");
        assert_eq!(parsed.market_slug.as_deref(), Some("some-market"));
    }

    #[test]
    fn parse_url_www_prefix() {
        let url = "https://www.polymarket.com/event/some-event";
        let parsed = parse_polymarket_url(url).unwrap();
        assert_eq!(parsed.event_slug, "some-event");
    }

    #[test]
    fn parse_url_www_no_scheme() {
        let url = "www.polymarket.com/event/some-event";
        let parsed = parse_polymarket_url(url).unwrap();
        assert_eq!(parsed.event_slug, "some-event");
    }

    #[test]
    fn parse_url_trailing_slash() {
        let url = "https://polymarket.com/event/some-event/";
        let parsed = parse_polymarket_url(url).unwrap();
        assert_eq!(parsed.event_slug, "some-event");
        assert_eq!(parsed.market_slug, None);
    }

    #[test]
    fn parse_url_trailing_slash_with_market() {
        let url = "https://polymarket.com/event/some-event/some-market/";
        let parsed = parse_polymarket_url(url).unwrap();
        assert_eq!(parsed.event_slug, "some-event");
        assert_eq!(parsed.market_slug.as_deref(), Some("some-market"));
    }

    #[test]
    fn parse_url_with_query_string() {
        let url = "https://polymarket.com/event/some-event/some-market?tid=abc123";
        let parsed = parse_polymarket_url(url).unwrap();
        assert_eq!(parsed.event_slug, "some-event");
        assert_eq!(parsed.market_slug.as_deref(), Some("some-market"));
    }

    #[test]
    fn parse_url_with_fragment() {
        let url = "https://polymarket.com/event/some-event#comments";
        let parsed = parse_polymarket_url(url).unwrap();
        assert_eq!(parsed.event_slug, "some-event");
    }

    #[test]
    fn parse_url_with_query_and_fragment() {
        let url = "https://polymarket.com/event/some-event/some-market?tid=1#top";
        let parsed = parse_polymarket_url(url).unwrap();
        assert_eq!(parsed.event_slug, "some-event");
        assert_eq!(parsed.market_slug.as_deref(), Some("some-market"));
    }

    #[test]
    fn parse_url_extra_path_segments_ignored() {
        let url = "https://polymarket.com/event/my-event/my-market/extra/stuff";
        let parsed = parse_polymarket_url(url).unwrap();
        assert_eq!(parsed.event_slug, "my-event");
        assert_eq!(parsed.market_slug.as_deref(), Some("my-market"));
    }

    #[test]
    fn parse_url_rejects_non_polymarket_domain() {
        assert!(parse_polymarket_url("https://example.com/event/foo").is_none());
        assert!(parse_polymarket_url("https://notpolymarket.com/event/foo").is_none());
    }

    #[test]
    fn parse_url_rejects_missing_event_prefix() {
        assert!(parse_polymarket_url("https://polymarket.com/markets/foo").is_none());
        assert!(parse_polymarket_url("https://polymarket.com/foo").is_none());
    }

    #[test]
    fn parse_url_rejects_empty_slug() {
        assert!(parse_polymarket_url("https://polymarket.com/event/").is_none());
    }

    #[test]
    fn parse_url_rejects_plain_slug() {
        assert!(parse_polymarket_url("will-bitcoin-hit-100k").is_none());
    }

    #[test]
    fn parse_url_rejects_numeric_id() {
        assert!(parse_polymarket_url("12345").is_none());
    }

    #[test]
    fn parse_url_rejects_no_path() {
        assert!(parse_polymarket_url("https://polymarket.com").is_none());
        assert!(parse_polymarket_url("polymarket.com").is_none());
    }

    // ── resolve_id ─────────────────────────────────────────────────

    #[test]
    fn resolve_id_numeric() {
        assert_eq!(
            resolve_id("12345", false),
            ResolvedId::Numeric("12345".to_string())
        );
        assert_eq!(
            resolve_id("12345", true),
            ResolvedId::Numeric("12345".to_string())
        );
    }

    #[test]
    fn resolve_id_plain_slug() {
        assert_eq!(
            resolve_id("will-bitcoin-hit-100k", false),
            ResolvedId::Slug("will-bitcoin-hit-100k".to_string())
        );
    }

    #[test]
    fn resolve_id_url_prefer_market_true() {
        let url = "https://polymarket.com/event/my-event/my-market";
        assert_eq!(
            resolve_id(url, true),
            ResolvedId::Slug("my-market".to_string())
        );
    }

    #[test]
    fn resolve_id_url_prefer_market_false() {
        let url = "https://polymarket.com/event/my-event/my-market";
        assert_eq!(
            resolve_id(url, false),
            ResolvedId::Slug("my-event".to_string())
        );
    }

    #[test]
    fn resolve_id_url_no_market_prefer_market_true() {
        let url = "https://polymarket.com/event/my-event";
        assert_eq!(
            resolve_id(url, true),
            ResolvedId::Slug("my-event".to_string())
        );
    }
}
