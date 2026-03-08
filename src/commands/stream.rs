//! `polymarket stream` — real-time WebSocket data streamed to the terminal.
//!
//! Uses a native WebSocket connection (via `tokio-tungstenite`) to the
//! Polymarket CLOB WebSocket API. No third-party SDK integration.

use anyhow::{Context as _, Result};
use clap::{Args, Subcommand};
use futures_util::StreamExt as _;

use crate::output::OutputFormat;
use crate::output::stream as stream_output;
use crate::ws;

#[derive(Args)]
pub struct StreamArgs {
    #[command(subcommand)]
    pub command: StreamCommand,

    /// Maximum number of events to receive before exiting (default: unlimited)
    #[arg(long, global = true)]
    pub max_events: Option<u64>,
}

#[derive(Subcommand)]
pub enum StreamCommand {
    /// Stream real-time orderbook snapshots (bids/asks)
    Orderbook {
        /// Token/asset IDs (comma-separated)
        token_ids: String,
    },
    /// Stream real-time price changes
    Prices {
        /// Token/asset IDs (comma-separated)
        token_ids: String,
    },
    /// Stream last trade price updates
    LastTrade {
        /// Token/asset IDs (comma-separated)
        token_ids: String,
    },
    /// Stream calculated midpoint prices
    Midpoints {
        /// Token/asset IDs (comma-separated)
        token_ids: String,
    },
}

fn split_ids(csv: &str) -> Vec<String> {
    csv.split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

pub async fn execute(args: StreamArgs, output: OutputFormat) -> Result<()> {
    let max = args.max_events;

    let (asset_ids, filter) = match &args.command {
        StreamCommand::Orderbook { token_ids } => (split_ids(token_ids), "book"),
        StreamCommand::Prices { token_ids } => (split_ids(token_ids), "price_change"),
        StreamCommand::LastTrade { token_ids } => (split_ids(token_ids), "last_trade_price"),
        StreamCommand::Midpoints { token_ids } => (split_ids(token_ids), "midpoint"),
    };

    anyhow::ensure!(!asset_ids.is_empty(), "At least one token ID is required");

    let mut stream = Box::pin(ws::subscribe_market(&asset_ids).await?);

    let mut count: u64 = 0;
    loop {
        tokio::select! {
            biased;

            _ = tokio::signal::ctrl_c() => {
                break;
            }

            frame = stream.next() => match frame {
                Some(result) => {
                    let event = result.context("WebSocket stream error")?;

                    // Filter to the requested event type
                    if event.event_type != filter {
                        continue;
                    }

                    // Check event limit before printing
                    if max.is_some_and(|m| count >= m) {
                        break;
                    }

                    stream_output::print_event(&event, &output)?;
                    count += 1;
                }
                None => break, // stream ended
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_ids_basic() {
        assert_eq!(split_ids("a,b,c"), vec!["a", "b", "c"]);
    }

    #[test]
    fn split_ids_trims_whitespace() {
        assert_eq!(split_ids(" a , b , c "), vec!["a", "b", "c"]);
    }

    #[test]
    fn split_ids_filters_empty() {
        assert_eq!(split_ids("a,,b, ,c"), vec!["a", "b", "c"]);
    }

    #[test]
    fn split_ids_single() {
        assert_eq!(split_ids("abc"), vec!["abc"]);
    }

    #[test]
    fn split_ids_empty_string() {
        assert!(split_ids("").is_empty());
    }
}
