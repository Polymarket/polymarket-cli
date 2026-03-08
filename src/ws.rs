//! Native WebSocket client for the Polymarket CLOB real-time API.
//!
//! Connects directly to `wss://ws-subscriptions-clob.polymarket.com/ws/market`
//! using `tokio-tungstenite`. No SDK dependency — just raw WebSocket frames.

use std::str;

use anyhow::{anyhow, Context, Result};
use futures_util::{Stream, SinkExt as _, StreamExt as _};
use rustls::crypto::ring::default_provider;
use serde::{Deserialize, Serialize};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

const WS_MARKET_URL: &str = "wss://ws-subscriptions-clob.polymarket.com/ws/market";

// ---------------------------------------------------------------------------
// Subscription protocol
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct SubscribeRequest<'a> {
    r#type: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    operation: Option<&'static str>,
    #[serde(rename = "assets_ids")]
    asset_ids: &'a [String],
    markets: &'a [String],
    #[serde(skip_serializing_if = "Option::is_none")]
    initial_dump: Option<bool>,
}

impl<'a> SubscribeRequest<'a> {
    fn market(asset_ids: &'a [String]) -> Self {
        Self {
            r#type: "market",
            operation: Some("subscribe"),
            asset_ids,
            markets: &[],
            initial_dump: Some(true),
        }
    }
}

// ---------------------------------------------------------------------------
// Inbound message types (only what the CLI needs)
// ---------------------------------------------------------------------------

/// Raw JSON event from the WebSocket — we keep it loosely typed so the CLI
/// never breaks when the upstream adds new fields.
#[derive(Debug, Clone, Deserialize)]
pub struct WsEvent {
    pub event_type: String,
    #[serde(flatten)]
    pub payload: serde_json::Value,
}

// ---------------------------------------------------------------------------
// Connection
// ---------------------------------------------------------------------------

/// Connect to the market channel, subscribe to the given asset IDs, and yield
/// each raw [`WsEvent`] to the caller.
///
/// The returned stream is live: it stays open until the server closes it, the
/// caller drops the stream, or a fatal error occurs.
pub async fn subscribe_market(
    asset_ids: &[String],
) -> Result<impl Stream<Item = Result<WsEvent>>> {
    // Install the default crypto provider for rustls (ignores errors if already installed)
    let _ = default_provider().install_default();

    let (ws_stream, _response) = connect_async(WS_MARKET_URL)
        .await
        .context("Failed to connect to Polymarket WebSocket")?;

    let (mut sink, stream) = ws_stream.split();

    // Send subscription
    let request = SubscribeRequest::market(asset_ids);
    let payload = serde_json::to_string(&request)?;
    sink.send(Message::Text(payload.into())).await?;

    // Map incoming frames → WsEvent(s)
    Ok(stream.filter_map(move |frame_result| async move {
        let frame = match frame_result {
            Ok(f) => f,
            Err(e) => return Some(Err(anyhow!("WebSocket error: {e}"))),
        };

        let text = match &frame {
            Message::Text(t) => t.as_ref(),
            Message::Binary(b) => {
                return match str::from_utf8(b) {
                    Ok(s) => parse_events(s),
                    Err(_) => None,
                }
            }
            Message::Ping(_) | Message::Pong(_) | Message::Frame(_) => return None,
            Message::Close(_) => return Some(Err(anyhow!("WebSocket closed by server"))),
        };

        parse_events(text)
    }))
}

/// Parse a JSON text frame into a single `WsEvent` result.
///
/// The server may send single objects or arrays — we handle both.
fn parse_events(text: &str) -> Option<Result<WsEvent>> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Single object
    if trimmed.starts_with('{') {
        return match serde_json::from_str::<WsEvent>(trimmed) {
            Ok(event) => Some(Ok(event)),
            Err(e) => Some(Err(anyhow!("Failed to parse WS message: {e}"))),
        };
    }

    // Array — yield the first parseable event (the stream adapter is per-frame)
    if trimmed.starts_with('[') {
        return match serde_json::from_str::<Vec<WsEvent>>(trimmed) {
            Ok(events) => events.into_iter().next().map(Ok),
            Err(e) => Some(Err(anyhow!("Failed to parse WS array message: {e}"))),
        };
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_single_event() {
        let json = r#"{"event_type":"book","asset_id":"123","market":"0x01","timestamp":"1234567890","bids":[],"asks":[]}"#;
        let result = parse_events(json);
        assert!(result.is_some());
        let event = result.unwrap().unwrap();
        assert_eq!(event.event_type, "book");
    }

    #[test]
    fn parse_array_events() {
        let json = r#"[{"event_type":"price_change","market":"0x01","timestamp":"123","price_changes":[]}]"#;
        let result = parse_events(json);
        assert!(result.is_some());
        let event = result.unwrap().unwrap();
        assert_eq!(event.event_type, "price_change");
    }

    #[test]
    fn parse_empty_returns_none() {
        assert!(parse_events("").is_none());
        assert!(parse_events("  ").is_none());
    }

    #[test]
    fn subscribe_request_serialises_correctly() {
        let ids = vec!["123".to_string(), "456".to_string()];
        let req = SubscribeRequest::market(&ids);
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains(r#""type":"market"#));
        assert!(json.contains(r#""operation":"subscribe"#));
        assert!(json.contains(r#""assets_ids":["123","456"]"#));
        assert!(json.contains(r#""initial_dump":true"#));
    }

    #[test]
    fn parse_malformed_array_returns_error() {
        let json = r#"[{"not_an_event": true}]"#;
        let result = parse_events(json);
        assert!(result.is_some());
        assert!(result.unwrap().is_err());
    }

    #[test]
    fn parse_non_json_text_returns_none() {
        assert!(parse_events("hello world").is_none());
    }

    #[test]
    fn parse_empty_array_returns_none() {
        let result = parse_events("[]");
        assert!(result.is_some_and(|r| r.is_ok()) == false);
        // Empty array parses fine but has no elements → None from .next()
        assert!(parse_events("[]").is_none());
    }
}
