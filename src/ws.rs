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

    // Map incoming frames → WsEvent(s), flattening arrays into individual items
    Ok(stream.flat_map(move |frame_result| {
        let events: Vec<Result<WsEvent>> = match frame_result {
            Err(e) => vec![Err(anyhow!("WebSocket error: {e}"))],
            Ok(frame) => match &frame {
                Message::Text(t) => parse_events(t.as_ref()),
                Message::Binary(b) => match str::from_utf8(b) {
                    Ok(s) => parse_events(s),
                    Err(_) => vec![],
                },
                Message::Close(_) => vec![Err(anyhow!("WebSocket closed by server"))],
                _ => vec![],
            },
        };
        futures_util::stream::iter(events)
    }))
}

/// Parse a JSON text frame into zero or more [`WsEvent`] results.
///
/// The server may send single objects or arrays — we handle both and
/// return **all** events so that array frames are never silently truncated.
fn parse_events(text: &str) -> Vec<Result<WsEvent>> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return vec![];
    }

    // Single object
    if trimmed.starts_with('{') {
        return match serde_json::from_str::<WsEvent>(trimmed) {
            Ok(event) => vec![Ok(event)],
            Err(e) => vec![Err(anyhow!("Failed to parse WS message: {e}"))],
        };
    }

    // Array — yield every event in the array
    if trimmed.starts_with('[') {
        return match serde_json::from_str::<Vec<WsEvent>>(trimmed) {
            Ok(events) => events.into_iter().map(Ok).collect(),
            Err(e) => vec![Err(anyhow!("Failed to parse WS array message: {e}"))],
        };
    }

    vec![]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_single_event() {
        let json = r#"{"event_type":"book","asset_id":"123","market":"0x01","timestamp":"1234567890","bids":[],"asks":[]}"#;
        let results = parse_events(json);
        assert_eq!(results.len(), 1);
        let event = results.into_iter().next().unwrap().unwrap();
        assert_eq!(event.event_type, "book");
    }

    #[test]
    fn parse_array_single_event() {
        let json = r#"[{"event_type":"price_change","market":"0x01","timestamp":"123","price_changes":[]}]"#;
        let results = parse_events(json);
        assert_eq!(results.len(), 1);
        let event = results.into_iter().next().unwrap().unwrap();
        assert_eq!(event.event_type, "price_change");
    }

    #[test]
    fn parse_array_yields_all_events() {
        let json = r#"[
            {"event_type":"price_change","market":"0x01","timestamp":"1","price_changes":[]},
            {"event_type":"book","market":"0x02","timestamp":"2","bids":[],"asks":[]},
            {"event_type":"midpoint","market":"0x03","timestamp":"3","midpoint":"0.5"}
        ]"#;
        let results = parse_events(json);
        assert_eq!(results.len(), 3, "All events in the array must be yielded");
        let types: Vec<String> = results
            .into_iter()
            .map(|r| r.unwrap().event_type)
            .collect();
        assert_eq!(types, vec!["price_change", "book", "midpoint"]);
    }

    #[test]
    fn parse_empty_returns_empty_vec() {
        assert!(parse_events("").is_empty());
        assert!(parse_events("  ").is_empty());
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
        let results = parse_events(json);
        assert_eq!(results.len(), 1);
        assert!(results.into_iter().next().unwrap().is_err());
    }

    #[test]
    fn parse_non_json_text_returns_empty() {
        assert!(parse_events("hello world").is_empty());
    }

    #[test]
    fn parse_empty_array_returns_empty() {
        let results = parse_events("[]");
        assert!(results.is_empty());
    }
}
