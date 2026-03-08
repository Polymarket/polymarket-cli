//! Minimal standalone WebSocket test — connects to Polymarket and dumps raw frames.
//! Run with: cargo run --example ws_test

use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use rustls::crypto::ring::default_provider;
use tokio::time::sleep;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

const WS_URL: &str = "wss://ws-subscriptions-clob.polymarket.com/ws/market";

#[tokio::main]
async fn main() {
    let _ = default_provider().install_default();

    println!("Connecting to {WS_URL}...");
    let (ws, resp) = connect_async(WS_URL).await.expect("Failed to connect");
    println!("Connected! Status: {}", resp.status());

    let (mut sink, mut stream) = ws.split();

    // Subscribe to known active market tokens
    let sub = serde_json::json!({
        "type": "market",
        "operation": "subscribe",
        "assets_ids": ["38397507750621893057346880033441136112987238933685677349709401910643842844855", "95949957895141858444199258452803633110472396604599808168788254125381075552218"],
        "markets": [],
        "initial_dump": true
    });
    let payload = serde_json::to_string(&sub).unwrap();
    println!("Sending: {payload}");
    sink.send(Message::Text(payload.into())).await.unwrap();

    println!("Waiting for frames (up to 30 seconds)...");
    let mut count = 0;
    let timeout = sleep(Duration::from_secs(30));
    tokio::pin!(timeout);  // macro — must remain fully qualified

    loop {
        tokio::select! {
            msg = stream.next() => {
                match msg {
                    Some(Ok(Message::Text(t))) => {
                        let preview: String = if t.chars().count() > 200 {
                            format!("{}...", t.chars().take(200).collect::<String>())
                        } else {
                            t.to_string()
                        };
                        println!("[TEXT #{count}] {preview}");
                        count += 1;
                    }
                    Some(Ok(Message::Binary(b))) => {
                        let s = String::from_utf8_lossy(&b);
                        let preview: String = if s.chars().count() > 200 {
                            format!("{}...", s.chars().take(200).collect::<String>())
                        } else {
                            s.to_string()
                        };
                        println!("[BIN #{count}] {preview}");
                        count += 1;
                    }
                    Some(Ok(Message::Ping(p))) => println!("[PING] {} bytes", p.len()),
                    Some(Ok(Message::Pong(p))) => println!("[PONG] {} bytes", p.len()),
                    Some(Ok(Message::Close(c))) => {
                        println!("[CLOSE] {c:?}");
                        break;
                    }
                    Some(Ok(Message::Frame(f))) => println!("[FRAME] {f:?}"),
                    Some(Err(e)) => {
                        eprintln!("[ERROR] {e}");
                        break;
                    }
                    None => {
                        println!("Stream ended.");
                        break;
                    }
                }
                if count >= 10 {
                    println!("Got 10 frames, exiting.");
                    break;
                }
            }
            _ = &mut timeout => {
                println!("Timeout after 30 seconds. Received {count} frames total.");
                break;
            }
        }
    }
}
