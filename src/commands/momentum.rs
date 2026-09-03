use std::str::FromStr;
use std::time::Duration;

use anyhow::{Context, Result};
use chrono::Utc;
use clap::{Args, Subcommand, ValueEnum};
use futures_util::{SinkExt, StreamExt};
use polymarket_client_sdk_v2::clob::types::request::PriceRequest;
use polymarket_client_sdk_v2::clob::types::{Amount, OrderType, Side};
use polymarket_client_sdk_v2::gamma::{self, types::request::MarketBySlugRequest};
use polymarket_client_sdk_v2::types::{Decimal, U256};
use serde::Deserialize;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;

use crate::auth;
use crate::output::OutputFormat;

// Coinbase's public trade feed as a free, no-auth stand-in for Chainlink's BTC/USD Data
// Streams feed (which these markets actually resolve against but which requires a paid
// Chainlink Data Streams subscription we don't have). Coinbase is one of the exchanges
// Chainlink itself aggregates from, so it tracks the same underlying price, just without
// Chainlink's own aggregation/verification hop — sometimes ahead of it, sometimes behind.
const COINBASE_WS_URL: &str = "wss://ws-feed.exchange.coinbase.com";
const COINBASE_PRODUCT: &str = "BTC-USD";

#[derive(Args)]
pub struct MomentumArgs {
    #[command(subcommand)]
    pub command: MomentumCommand,
}

#[derive(Subcommand)]
pub enum MomentumCommand {
    /// Watch Polymarket's BTC 5m/15m Chainlink-resolved Up/Down markets and buy the side
    /// a fast exchange price feed favors, before Polymarket's own order book catches up.
    ///
    /// This is a directional-noise heuristic, not a resolution guarantee: the market
    /// resolves against Chainlink's BTC/USD TWAP, this tool watches Coinbase spot as a
    /// proxy, and the two can diverge. Trades live by default — use --dry-run to only
    /// print signals.
    Run {
        /// Market window: 5m or 15m
        #[arg(long)]
        window: CliWindow,

        /// USDC stake per entry (one entry per window)
        #[arg(long)]
        stake: String,

        /// Minimum price move from the window-open reference, in basis points, before a
        /// signal is considered (filters out bid/ask noise)
        #[arg(long, default_value = "5")]
        min_deviation_bps: u32,

        /// Seconds to wait after a window opens before acting on a signal
        #[arg(long, default_value = "3")]
        settle_in_secs: i64,

        /// Don't enter within this many seconds of window close
        #[arg(long, default_value = "15")]
        min_seconds_remaining: i64,

        /// Skip entry if the favored side already costs more than this (0-1)
        #[arg(long, default_value = "0.85")]
        max_entry_price: String,

        /// Stop after this many completed windows (default: run until interrupted)
        #[arg(long)]
        max_windows: Option<u32>,

        /// Only print signals — never places real orders
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum CliWindow {
    #[value(name = "5m")]
    FiveMin,
    #[value(name = "15m")]
    FifteenMin,
}

impl CliWindow {
    const fn seconds(self) -> i64 {
        match self {
            Self::FiveMin => 300,
            Self::FifteenMin => 900,
        }
    }

    const fn slug_part(self) -> &'static str {
        match self {
            Self::FiveMin => "5m",
            Self::FifteenMin => "15m",
        }
    }
}

struct RunConfig {
    window: CliWindow,
    stake: Decimal,
    min_deviation_bps: u32,
    settle_in_secs: i64,
    min_seconds_remaining: i64,
    max_entry_price: Decimal,
    max_windows: Option<u32>,
    dry_run: bool,
}

struct ActiveMarket {
    slug: String,
    outcomes: Vec<String>,
    token_ids: Vec<U256>,
}

impl ActiveMarket {
    fn token_for_outcome(&self, outcome: &str) -> Option<U256> {
        self.outcomes
            .iter()
            .position(|o| o.eq_ignore_ascii_case(outcome))
            .and_then(|i| self.token_ids.get(i).copied())
    }
}

#[derive(Deserialize)]
struct CoinbaseTicker {
    #[serde(rename = "type")]
    msg_type: String,
    price: Option<String>,
}

pub async fn execute(
    args: MomentumArgs,
    output: OutputFormat,
    private_key: Option<&str>,
    signature_type: Option<&str>,
) -> Result<()> {
    match args.command {
        MomentumCommand::Run {
            window,
            stake,
            min_deviation_bps,
            settle_in_secs,
            min_seconds_remaining,
            max_entry_price,
            max_windows,
            dry_run,
        } => {
            let stake = Decimal::from_str(&stake)
                .map_err(|_| anyhow::anyhow!("Invalid stake: {stake}"))?;
            anyhow::ensure!(stake > Decimal::ZERO, "stake must be greater than 0");
            let max_entry_price = Decimal::from_str(&max_entry_price)
                .map_err(|_| anyhow::anyhow!("Invalid max-entry-price: {max_entry_price}"))?;

            let cfg = RunConfig {
                window,
                stake,
                min_deviation_bps,
                settle_in_secs,
                min_seconds_remaining,
                max_entry_price,
                max_windows,
                dry_run,
            };
            run(cfg, output, private_key, signature_type).await
        }
    }
}

fn log(output: OutputFormat, message: &str) {
    match output {
        OutputFormat::Table => println!("[{}] {message}", Utc::now().format("%H:%M:%S")),
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::json!({"ts": Utc::now().to_rfc3339(), "message": message})
            );
        }
    }
}

async fn run(
    cfg: RunConfig,
    output: OutputFormat,
    private_key: Option<&str>,
    signature_type: Option<&str>,
) -> Result<()> {
    let gamma = gamma::Client::default();
    let unauth = auth::unauthenticated_clob_client()?;

    // Only require a funded/authenticated wallet when we might actually place orders.
    let trader = if cfg.dry_run {
        None
    } else {
        let signer = auth::resolve_signer(private_key)?;
        let client = auth::authenticate_with_signer(&signer, signature_type).await?;
        Some((signer, client))
    };

    log(
        output,
        &format!(
            "Watching BTC {} Chainlink-resolved markets via Coinbase BTC-USD as the fast \
             proxy feed. {}",
            cfg.window.slug_part(),
            if cfg.dry_run {
                "DRY RUN — no orders will be placed.".to_string()
            } else {
                format!(
                    "LIVE — {} USDC per entry, real orders will be placed.",
                    cfg.stake
                )
            }
        ),
    );

    let (tx, mut rx) = mpsc::unbounded_channel::<Decimal>();
    tokio::spawn(run_price_feed(tx));

    let window_secs = cfg.window.seconds();
    let mut current_slot: Option<i64> = None;
    let mut reference_price: Option<Decimal> = None;
    let mut current_market: Option<ActiveMarket> = None;
    let mut traded_this_window = false;
    let mut warming_up = true;
    let mut windows_completed: u32 = 0;

    while let Some(price) = rx.recv().await {
        let now = Utc::now().timestamp();
        let slot = now - now.rem_euclid(window_secs);

        if current_slot != Some(slot) {
            if current_slot.is_some() {
                windows_completed += 1;
                if let Some(max) = cfg.max_windows
                    && windows_completed >= max
                {
                    log(output, "Reached --max-windows, stopping.");
                    break;
                }
            } else {
                log(
                    output,
                    "Started mid-window — skipping it to get a clean reference price at the \
                     next window boundary.",
                );
            }

            current_slot = Some(slot);
            traded_this_window = false;

            if warming_up {
                warming_up = false;
                reference_price = None;
                current_market = None;
            } else {
                reference_price = Some(price);
                current_market = match fetch_active_market(&gamma, cfg.window, slot).await {
                    Ok(market) => {
                        log(
                            output,
                            &format!("New window {} — reference price ${price}", market.slug),
                        );
                        Some(market)
                    }
                    Err(e) => {
                        eprintln!("[momentum] failed to load market for this window: {e}");
                        None
                    }
                };
            }
        }

        let (Some(reference), Some(market)) = (reference_price, current_market.as_ref()) else {
            continue;
        };
        if traded_this_window {
            continue;
        }

        let seconds_since_open = now - slot;
        let seconds_remaining = slot + window_secs - now;
        if seconds_since_open < cfg.settle_in_secs || seconds_remaining < cfg.min_seconds_remaining
        {
            continue;
        }

        let deviation_bps = (price - reference) / reference * Decimal::from(10_000);
        if deviation_bps.abs() < Decimal::from(cfg.min_deviation_bps) {
            continue;
        }

        let favored = if deviation_bps > Decimal::ZERO {
            "Up"
        } else {
            "Down"
        };
        let Some(token_id) = market.token_for_outcome(favored) else {
            eprintln!("[momentum] market has no \"{favored}\" outcome token");
            continue;
        };

        let ask = match best_ask(&unauth, token_id).await {
            Ok(a) => a,
            Err(e) => {
                eprintln!("[momentum] failed to fetch current price: {e}");
                continue;
            }
        };
        if ask > cfg.max_entry_price {
            continue;
        }

        log(
            output,
            &format!(
                "Signal: {favored} favored ({deviation_bps:+.1} bps vs ref ${reference}), ask \
                 {ask}, {seconds_remaining}s left in window"
            ),
        );

        if let Some((signer, client)) = &trader {
            match place_market_buy(client, signer, token_id, cfg.stake).await {
                Ok((order_id, status)) => {
                    log(
                        output,
                        &format!(
                            "Bought {} USDC of {favored} — order {order_id} ({status})",
                            cfg.stake
                        ),
                    );
                }
                Err(e) => eprintln!("[momentum] order failed: {e}"),
            }
        } else {
            log(
                output,
                &format!("[dry-run] would buy {} USDC of {favored} @ ~{ask}", cfg.stake),
            );
        }
        traded_this_window = true;
    }

    Ok(())
}

async fn fetch_active_market(
    gamma: &gamma::Client,
    window: CliWindow,
    slot: i64,
) -> Result<ActiveMarket> {
    let slug = format!("btc-updown-{}-{slot}", window.slug_part());
    let request = MarketBySlugRequest::builder().slug(slug.clone()).build();
    let market = gamma
        .market_by_slug(&request)
        .await
        .with_context(|| format!("Failed to fetch market {slug}"))?;
    let outcomes = market
        .outcomes
        .with_context(|| format!("Market {slug} has no outcomes"))?;
    let token_ids = market
        .clob_token_ids
        .with_context(|| format!("Market {slug} has no CLOB token ids"))?;
    anyhow::ensure!(
        outcomes.len() == token_ids.len(),
        "Market {slug} outcome/token count mismatch"
    );
    Ok(ActiveMarket {
        slug,
        outcomes,
        token_ids,
    })
}

async fn best_ask(client: &polymarket_client_sdk_v2::clob::Client, token_id: U256) -> Result<Decimal> {
    let request = PriceRequest::builder().token_id(token_id).side(Side::Buy).build();
    let result = client.price(&request).await?;
    Ok(result.price)
}

async fn place_market_buy(
    client: &polymarket_client_sdk_v2::clob::Client<
        polymarket_client_sdk_v2::auth::state::Authenticated<polymarket_client_sdk_v2::auth::Normal>,
    >,
    signer: &(impl polymarket_client_sdk_v2::auth::Signer + Sync),
    token_id: U256,
    stake: Decimal,
) -> Result<(String, String)> {
    let amount = Amount::usdc(stake)?;
    let order = client
        .market_order()
        .token_id(token_id)
        .side(Side::Buy)
        .amount(amount)
        .order_type(OrderType::FOK)
        .build()
        .await?;
    let signed_order = client.sign(signer, order).await?;
    let mut results = client.post_orders(vec![signed_order]).await?;
    let result = results
        .pop()
        .ok_or_else(|| anyhow::anyhow!("Order submission returned no result"))?;
    anyhow::ensure!(
        result.success,
        "{}",
        result
            .error_msg
            .filter(|m| !m.is_empty())
            .unwrap_or_else(|| "order was not accepted".to_string())
    );
    Ok((result.order_id, result.status.to_string()))
}

async fn run_price_feed(tx: mpsc::UnboundedSender<Decimal>) {
    loop {
        if let Err(e) = run_price_feed_once(&tx).await {
            eprintln!("[momentum] price feed error: {e}; reconnecting in 3s");
        }
        tokio::time::sleep(Duration::from_secs(3)).await;
    }
}

async fn run_price_feed_once(tx: &mpsc::UnboundedSender<Decimal>) -> Result<()> {
    let (ws_stream, _) = tokio_tungstenite::connect_async(COINBASE_WS_URL)
        .await
        .context("Failed to connect to Coinbase price feed")?;
    let (mut write, mut read) = ws_stream.split();

    let subscribe = serde_json::json!({
        "type": "subscribe",
        "product_ids": [COINBASE_PRODUCT],
        "channels": ["ticker"],
    });
    write
        .send(Message::Text(subscribe.to_string()))
        .await
        .context("Failed to subscribe to Coinbase price feed")?;

    while let Some(msg) = read.next().await {
        let msg = msg.context("Coinbase price feed connection error")?;
        let Ok(text) = msg.to_text() else { continue };
        let Ok(ticker) = serde_json::from_str::<CoinbaseTicker>(text) else {
            continue;
        };
        if ticker.msg_type != "ticker" {
            continue;
        }
        let Some(price_str) = ticker.price else { continue };
        let Ok(price) = Decimal::from_str(&price_str) else {
            continue;
        };
        if tx.send(price).is_err() {
            return Ok(());
        }
    }

    anyhow::bail!("Coinbase price feed stream ended")
}
