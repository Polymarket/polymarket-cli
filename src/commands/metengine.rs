use anyhow::{Result, bail};
use clap::{Args, Subcommand};
use polymarket_client_sdk::gamma::{self, types::request::MarketBySlugRequest};
use serde_json::json;

use crate::config;
use crate::metengine::{FreeClient, PaidClient};
use crate::output::OutputFormat;
use crate::output::metengine as out;

/// Resolve a market identifier to a condition ID.
///
/// Accepts:
/// - `0x...` hex condition ID (validated: 66 chars, hex digits only)
/// - `https://polymarket.com/event/.../slug` URL (extracts last path segment)
/// - bare slug (resolved via Gamma API)
async fn resolve_condition_id(input: &str) -> Result<String> {
    let trimmed = input.trim();

    // Already a hex condition ID -- validate format
    if trimmed.starts_with("0x") || trimmed.starts_with("0X") {
        let hex = &trimmed[2..];
        if hex.len() != 64 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
            bail!("Invalid condition ID: must be 0x followed by 64 hex characters");
        }
        return Ok(trimmed.to_lowercase());
    }

    // Extract slug from URL or use as bare slug
    let slug = if let Ok(url) = reqwest::Url::parse(trimmed) {
        let host = url.host_str().unwrap_or_default();
        if host != "polymarket.com" && !host.ends_with(".polymarket.com") {
            bail!("Only polymarket.com URLs are supported, got: {host}");
        }
        url.path_segments()
            .and_then(|segs| segs.filter(|s| !s.is_empty()).next_back())
            .ok_or_else(|| anyhow::anyhow!("Could not extract slug from URL"))?
            .to_string()
    } else {
        trimmed.to_string()
    };

    // Resolve slug via Gamma API
    let client = gamma::Client::default();
    let req = MarketBySlugRequest::builder().slug(&slug).build();
    let market = client
        .market_by_slug(&req)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to resolve slug '{slug}': {e}"))?;

    let cid = market
        .condition_id
        .ok_or_else(|| anyhow::anyhow!("Market '{slug}' has no condition_id"))?;

    Ok(cid.to_string().to_lowercase())
}

#[derive(Args)]
pub struct MetengineArgs {
    #[command(subcommand)]
    pub command: MetengineCommand,
}

#[derive(Subcommand)]
pub enum MetengineCommand {
    // -- Free endpoints --
    /// Check MetEngine API health
    Health,
    /// Show endpoint pricing tiers
    Pricing,

    // -- Market Discovery --
    /// Trending markets by volume spike, trade count, or smart money inflow
    Trending {
        /// Time window: 1h, 4h, 12h, 24h, 7d
        #[arg(long, default_value = "24h")]
        timeframe: String,
        /// Filter by category
        #[arg(long)]
        category: Option<String>,
        /// Sort: volume_spike, trade_count, smart_money_inflow
        #[arg(long, default_value = "volume_spike")]
        sort_by: String,
        /// Max results (1-100)
        #[arg(long, default_value = "20")]
        limit: u32,
    },
    /// Search markets by keyword or URL
    Search {
        /// Search query
        query: String,
        /// Filter by category
        #[arg(long)]
        category: Option<String>,
        /// Status: active, closing_soon, resolved
        #[arg(long, default_value = "active")]
        status: String,
        /// Only markets with smart money signal
        #[arg(long)]
        has_smart_money_signal: bool,
        /// Sort: relevance, volume, end_date
        #[arg(long, default_value = "relevance")]
        sort_by: String,
        /// Max results (1-100)
        #[arg(long, default_value = "20")]
        limit: u32,
    },
    /// List market categories with stats
    Categories {
        /// Include volume/trade stats
        #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
        include_stats: bool,
        /// Time window: 24h, 7d
        #[arg(long, default_value = "24h")]
        timeframe: String,
    },
    /// Platform-wide statistics
    PlatformStats {
        /// Time window: 24h, 7d, 30d
        #[arg(long, default_value = "24h")]
        timeframe: String,
    },
    /// OHLCV price history for a market
    PriceHistory {
        /// Market condition ID (0x...), slug, or Polymarket URL
        market: String,
        /// Time window: 1h, 4h, 12h, 24h, 7d, 30d
        #[arg(long, default_value = "7d")]
        timeframe: String,
        /// Candle size: 5m, 15m, 1h, 4h, 12h, 1d
        #[arg(long, default_value = "1h")]
        bucket_size: String,
    },
    /// Find markets with overlapping traders
    Similar {
        /// Market condition ID (0x...), slug, or Polymarket URL
        market: String,
        /// Max results (1-50)
        #[arg(long, default_value = "10")]
        limit: u32,
    },
    /// Smart money opportunity scanner
    Opportunities {
        /// Minimum signal: weak, moderate, strong
        #[arg(long, default_value = "moderate")]
        min_signal_strength: String,
        /// Filter by category
        #[arg(long)]
        category: Option<String>,
        /// Only markets closing within N hours
        #[arg(long)]
        closing_within_hours: Option<u32>,
        /// Minimum smart wallets
        #[arg(long, default_value = "3")]
        min_smart_wallets: u32,
        /// Max results (1-100)
        #[arg(long, default_value = "20")]
        limit: u32,
    },
    /// Markets with strongest smart money conviction
    HighConviction {
        /// Filter by category
        #[arg(long)]
        category: Option<String>,
        /// Minimum smart wallets
        #[arg(long, default_value = "5")]
        min_smart_wallets: u32,
        /// Minimum average smart score (0-100)
        #[arg(long, default_value = "65")]
        min_avg_score: u32,
        /// Max results (1-100)
        #[arg(long, default_value = "20")]
        limit: u32,
    },
    /// Capital flow across categories
    CapitalFlow {
        /// Time window: 24h, 7d, 30d
        #[arg(long, default_value = "7d")]
        timeframe: String,
        /// Smart money only
        #[arg(long)]
        smart_money_only: bool,
        /// Top N categories (1-50)
        #[arg(long, default_value = "20")]
        top_n_categories: u32,
    },
    /// Volume heatmap by category, hour, or day
    VolumeHeatmap {
        /// Time window: 24h, 7d, 30d
        #[arg(long, default_value = "24h")]
        timeframe: String,
        /// Group by: category, hour_of_day, day_of_week
        #[arg(long, default_value = "category")]
        group_by: String,
        /// Smart money only
        #[arg(long)]
        smart_money_only: bool,
    },
    /// Recently resolved markets with smart money accuracy
    Resolutions {
        /// Filter by category
        #[arg(long)]
        category: Option<String>,
        /// Keyword filter
        #[arg(long)]
        query: Option<String>,
        /// Sort: resolved_recently, volume, smart_money_accuracy
        #[arg(long, default_value = "resolved_recently")]
        sort_by: String,
        /// Max results (1-100)
        #[arg(long, default_value = "25")]
        limit: u32,
    },
    /// Dumb money positions on a market (low-score traders)
    DumbMoney {
        /// Market condition ID (0x...), slug, or Polymarket URL
        market: String,
        /// Max wallet score (0-100)
        #[arg(long, default_value = "30")]
        max_score: u32,
        /// Minimum trades
        #[arg(long, default_value = "5")]
        min_trades: u32,
        /// Max results (1-200)
        #[arg(long, default_value = "50")]
        limit: u32,
    },

    // -- Intelligence / Trades --
    /// Deep smart money intelligence for a market
    Intelligence {
        /// Market condition ID (0x...), slug, or Polymarket URL
        market: String,
        /// Top N wallets to analyze (1-50)
        #[arg(long, default_value = "10")]
        top_n_wallets: u32,
    },
    /// Market sentiment over time
    Sentiment {
        /// Market condition ID (0x...), slug, or Polymarket URL
        market: String,
        /// Time window: 24h, 7d, 30d
        #[arg(long, default_value = "7d")]
        timeframe: String,
        /// Bucket size: 1h, 4h, 12h, 1d
        #[arg(long, default_value = "4h")]
        bucket_size: String,
    },
    /// Market participant breakdown by tier
    Participants {
        /// Market condition ID (0x...), slug, or Polymarket URL
        market: String,
    },
    /// Detect insider trading patterns on a market
    Insiders {
        /// Market condition ID (0x...), slug, or Polymarket URL
        market: String,
        /// Max results (1-100)
        #[arg(long, default_value = "25")]
        limit: u32,
        /// Minimum insider score (0-100)
        #[arg(long, default_value = "20")]
        min_score: u32,
    },
    /// Recent trades on a market
    Trades {
        /// Market condition ID (0x...), slug, or Polymarket URL
        market: String,
        /// Time window: 1h, 4h, 12h, 24h, 7d, 30d
        #[arg(long, default_value = "24h")]
        timeframe: String,
        /// Filter: BUY or SELL
        #[arg(long)]
        side: Option<String>,
        /// Minimum USDC size
        #[arg(long)]
        min_usdc: Option<f64>,
        /// Smart money only
        #[arg(long)]
        smart_money_only: bool,
        /// Max results (1-500)
        #[arg(long, default_value = "100")]
        limit: u32,
    },
    /// Large trades across all markets
    WhaleTrades {
        /// Minimum USDC size
        #[arg(long, default_value = "10000")]
        min_usdc: f64,
        /// Time window: 1h, 4h, 12h, 24h, 7d, 30d
        #[arg(long, default_value = "24h")]
        timeframe: String,
        /// Filter by market condition ID (0x...), slug, or Polymarket URL
        #[arg(long)]
        market: Option<String>,
        /// Filter by category
        #[arg(long)]
        category: Option<String>,
        /// Filter: BUY or SELL
        #[arg(long)]
        side: Option<String>,
        /// Smart money only
        #[arg(long)]
        smart_money_only: bool,
        /// Max results (1-200)
        #[arg(long, default_value = "50")]
        limit: u32,
    },
    /// Wallets that called market outcomes early
    AlphaCallers {
        /// Look back N days (1-90)
        #[arg(long, default_value = "30")]
        days_back: u32,
        /// Minimum days before resolution (1-60)
        #[arg(long, default_value = "7")]
        min_days_early: u32,
        /// Minimum bet size in USDC
        #[arg(long, default_value = "100")]
        min_bet_usdc: f64,
        /// Max results (1-100)
        #[arg(long, default_value = "25")]
        limit: u32,
    },
    /// Platform-wide insider detection
    WalletInsiders {
        /// Max results (1-200)
        #[arg(long, default_value = "50")]
        limit: u32,
        /// Minimum insider score (0-100)
        #[arg(long, default_value = "50")]
        min_score: u32,
        /// Maximum wallet age in days (1-90)
        #[arg(long, default_value = "60")]
        max_wallet_age_days: u32,
    },

    // -- Wallet Analytics --
    /// Full wallet profile with score, stats, positions
    WalletProfile {
        /// Wallet address (0x..., will be lowercased)
        wallet: String,
        /// Include active positions
        #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
        include_positions: bool,
        /// Include recent trades
        #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
        include_trades: bool,
        /// Trade history limit (1-500)
        #[arg(long, default_value = "50")]
        trades_limit: u32,
    },
    /// Recent wallet trading activity
    WalletActivity {
        /// Wallet address (0x..., will be lowercased)
        wallet: String,
        /// Time window: 1h, 4h, 24h, 7d, 30d
        #[arg(long, default_value = "24h")]
        timeframe: String,
        /// Filter by category
        #[arg(long)]
        category: Option<String>,
        /// Minimum USDC size
        #[arg(long)]
        min_usdc: Option<f64>,
        /// Max results (1-500)
        #[arg(long, default_value = "100")]
        limit: u32,
    },
    /// Wallet PnL breakdown by position
    WalletPnl {
        /// Wallet address (0x..., will be lowercased)
        wallet: String,
        /// Time window: 7d, 30d, 90d, all
        #[arg(long, default_value = "90d")]
        timeframe: String,
        /// Max positions (1-200)
        #[arg(long, default_value = "50")]
        limit: u32,
    },
    /// Compare 2-5 wallets side by side
    WalletCompare {
        /// Wallet addresses (2-5, comma-separated)
        wallets: String,
        /// Include shared positions
        #[arg(long)]
        include_shared_positions: bool,
    },
    /// Find wallets that copy-trade a target wallet
    CopyTraders {
        /// Target wallet address (0x..., will be lowercased)
        wallet: String,
        /// Max lag in minutes (1-1440)
        #[arg(long, default_value = "60")]
        max_lag_minutes: u32,
        /// Time window: 24h, 7d, 30d
        #[arg(long, default_value = "7d")]
        timeframe: String,
        /// Minimum overlapping trades
        #[arg(long, default_value = "3")]
        min_overlap_trades: u32,
        /// Max results (1-100)
        #[arg(long, default_value = "20")]
        limit: u32,
    },
    /// Top performing wallets leaderboard
    TopPerformers {
        /// Time window: today, 24h, 7d, 30d, 90d, 365d
        #[arg(long, default_value = "7d")]
        timeframe: String,
        /// Filter by category
        #[arg(long)]
        category: Option<String>,
        /// Rank by: pnl, roi, sharpe, win_rate, volume
        #[arg(long, default_value = "pnl")]
        metric: String,
        /// Minimum trades
        #[arg(long, default_value = "5")]
        min_trades: u32,
        /// Max results (1-100)
        #[arg(long, default_value = "25")]
        limit: u32,
    },
    /// Category specialists with high sharpe/win rate
    NicheExperts {
        /// Category (required)
        category: String,
        /// Minimum trades in category
        #[arg(long, default_value = "10")]
        min_category_trades: u32,
        /// Sort: category_sharpe, category_pnl, category_volume
        #[arg(long, default_value = "category_sharpe")]
        sort_by: String,
        /// Max results (1-100)
        #[arg(long, default_value = "25")]
        limit: u32,
    },
}

pub async fn execute(
    args: MetengineArgs,
    output: OutputFormat,
    solana_key: Option<&str>,
) -> Result<()> {
    match args.command {
        MetengineCommand::Health | MetengineCommand::Pricing => {
            execute_free(args.command, &output).await
        }
        _ => {
            let key = solana_key
                .map(String::from)
                .or_else(|| config::resolve_solana_key(None).0);
            let Some(key) = key else {
                bail!(
                    "Solana key required for paid MetEngine endpoints.\n\
                     Set METENGINE_SOLANA_KEY env var or use --solana-key flag."
                );
            };
            let client = PaidClient::new(&key)?;
            execute_paid(&client, args.command, &output).await
        }
    }
}

async fn execute_free(command: MetengineCommand, output: &OutputFormat) -> Result<()> {
    let client = FreeClient::new();
    match command {
        MetengineCommand::Health => {
            let data = client.get("/health").await?;
            out::print_health(&data, output)
        }
        MetengineCommand::Pricing => {
            let data = client.get("/api/v1/pricing").await?;
            out::print_pricing(&data, output)
        }
        _ => unreachable!(),
    }
}

fn push_opt<'a>(q: &mut Vec<(&'a str, &'a str)>, key: &'a str, val: &'a Option<String>) {
    if let Some(v) = val {
        q.push((key, v.as_str()));
    }
}

async fn execute_paid(
    client: &PaidClient,
    command: MetengineCommand,
    output: &OutputFormat,
) -> Result<()> {
    match command {
        // -- Market Discovery (GET) --
        MetengineCommand::Trending {
            timeframe,
            category,
            sort_by,
            limit,
        } => {
            let limit_s = limit.to_string();
            let mut q = vec![
                ("timeframe", timeframe.as_str()),
                ("sort_by", sort_by.as_str()),
                ("limit", limit_s.as_str()),
            ];
            push_opt(&mut q, "category", &category);
            let data = client.get("/api/v1/markets/trending", &q).await?;
            out::print_trending(&data, output)
        }
        MetengineCommand::Search {
            query,
            category,
            status,
            has_smart_money_signal,
            sort_by,
            limit,
        } => {
            let limit_s = limit.to_string();
            let mut q = vec![
                ("query", query.as_str()),
                ("status", status.as_str()),
                ("sort_by", sort_by.as_str()),
                ("limit", limit_s.as_str()),
            ];
            push_opt(&mut q, "category", &category);
            if has_smart_money_signal {
                q.push(("has_smart_money_signal", "true"));
            }
            let data = client.get("/api/v1/markets/search", &q).await?;
            out::print_search(&data, output)
        }
        MetengineCommand::Categories {
            include_stats,
            timeframe,
        } => {
            let include_stats_s = include_stats.to_string();
            let q = vec![
                ("include_stats", include_stats_s.as_str()),
                ("timeframe", timeframe.as_str()),
            ];
            let data = client.get("/api/v1/markets/categories", &q).await?;
            out::print_categories(&data, output)
        }
        MetengineCommand::PlatformStats { timeframe } => {
            let q = vec![("timeframe", timeframe.as_str())];
            let data = client.get("/api/v1/platform/stats", &q).await?;
            out::print_platform_stats(&data, output)
        }
        MetengineCommand::PriceHistory {
            market,
            timeframe,
            bucket_size,
        } => {
            let condition_id = resolve_condition_id(&market).await?;
            let q = vec![
                ("condition_id", condition_id.as_str()),
                ("timeframe", timeframe.as_str()),
                ("bucket_size", bucket_size.as_str()),
            ];
            let data = client.get("/api/v1/markets/price-history", &q).await?;
            out::print_price_history(&data, output)
        }
        MetengineCommand::Similar {
            market,
            limit,
        } => {
            let condition_id = resolve_condition_id(&market).await?;
            let limit_s = limit.to_string();
            let q = vec![("condition_id", condition_id.as_str()), ("limit", &limit_s)];
            let data = client.get("/api/v1/markets/similar", &q).await?;
            out::print_similar(&data, output)
        }
        MetengineCommand::Opportunities {
            min_signal_strength,
            category,
            closing_within_hours,
            min_smart_wallets,
            limit,
        } => {
            let limit_s = limit.to_string();
            let msw_s = min_smart_wallets.to_string();
            let mut q = vec![
                ("min_signal_strength", min_signal_strength.as_str()),
                ("min_smart_wallets", msw_s.as_str()),
                ("limit", limit_s.as_str()),
            ];
            push_opt(&mut q, "category", &category);
            let cwh_s;
            if let Some(h) = closing_within_hours {
                cwh_s = h.to_string();
                q.push(("closing_within_hours", cwh_s.as_str()));
            }
            let data = client.get("/api/v1/markets/opportunities", &q).await?;
            out::print_opportunities(&data, output)
        }
        MetengineCommand::HighConviction {
            category,
            min_smart_wallets,
            min_avg_score,
            limit,
        } => {
            let limit_s = limit.to_string();
            let msw_s = min_smart_wallets.to_string();
            let mas_s = min_avg_score.to_string();
            let mut q = vec![
                ("min_smart_wallets", msw_s.as_str()),
                ("min_avg_score", mas_s.as_str()),
                ("limit", limit_s.as_str()),
            ];
            push_opt(&mut q, "category", &category);
            let data = client.get("/api/v1/markets/high-conviction", &q).await?;
            out::print_high_conviction(&data, output)
        }
        MetengineCommand::CapitalFlow {
            timeframe,
            smart_money_only,
            top_n_categories,
        } => {
            let tnc_s = top_n_categories.to_string();
            let mut q = vec![
                ("timeframe", timeframe.as_str()),
                ("top_n_categories", &tnc_s),
            ];
            if smart_money_only {
                q.push(("smart_money_only", "true"));
            }
            let data = client.get("/api/v1/markets/capital-flow", &q).await?;
            out::print_capital_flow(&data, output)
        }
        MetengineCommand::VolumeHeatmap {
            timeframe,
            group_by,
            smart_money_only,
        } => {
            let mut q = vec![
                ("timeframe", timeframe.as_str()),
                ("group_by", group_by.as_str()),
            ];
            if smart_money_only {
                q.push(("smart_money_only", "true"));
            }
            let data = client.get("/api/v1/markets/volume-heatmap", &q).await?;
            out::print_volume_heatmap(&data, output)
        }
        MetengineCommand::Resolutions {
            category,
            query,
            sort_by,
            limit,
        } => {
            let limit_s = limit.to_string();
            let mut q = vec![("sort_by", sort_by.as_str()), ("limit", limit_s.as_str())];
            push_opt(&mut q, "category", &category);
            push_opt(&mut q, "query", &query);
            let data = client.get("/api/v1/markets/resolutions", &q).await?;
            out::print_resolutions(&data, output)
        }
        MetengineCommand::DumbMoney {
            market,
            max_score,
            min_trades,
            limit,
        } => {
            let condition_id = resolve_condition_id(&market).await?;
            let limit_s = limit.to_string();
            let ms_s = max_score.to_string();
            let mt_s = min_trades.to_string();
            let q = vec![
                ("condition_id", condition_id.as_str()),
                ("max_score", &ms_s),
                ("min_trades", &mt_s),
                ("limit", &limit_s),
            ];
            let data = client.get("/api/v1/markets/dumb-money", &q).await?;
            out::print_dumb_money(&data, output)
        }

        // -- Intelligence / Trades (POST for most, GET for trades/whales/alpha/wallet-insiders) --
        MetengineCommand::Intelligence {
            market,
            top_n_wallets,
        } => {
            let condition_id = resolve_condition_id(&market).await?;
            let body = json!({
                "condition_id": condition_id,
                "top_n_wallets": top_n_wallets,
            });
            let data = client.post("/api/v1/markets/intelligence", &body).await?;
            out::print_intelligence(&data, output)
        }
        MetengineCommand::Sentiment {
            market,
            timeframe,
            bucket_size,
        } => {
            let condition_id = resolve_condition_id(&market).await?;
            let body = json!({
                "condition_id": condition_id,
                "timeframe": timeframe,
                "bucket_size": bucket_size,
            });
            let data = client.post("/api/v1/markets/sentiment", &body).await?;
            out::print_sentiment(&data, output)
        }
        MetengineCommand::Participants { market } => {
            let condition_id = resolve_condition_id(&market).await?;
            let body = json!({ "condition_id": condition_id });
            let data = client.post("/api/v1/markets/participants", &body).await?;
            out::print_participants(&data, output)
        }
        MetengineCommand::Insiders {
            market,
            limit,
            min_score,
        } => {
            let condition_id = resolve_condition_id(&market).await?;
            let body = json!({
                "condition_id": condition_id,
                "limit": limit,
                "min_score": min_score,
            });
            let data = client.post("/api/v1/markets/insiders", &body).await?;
            out::print_insiders(&data, output)
        }
        MetengineCommand::Trades {
            market,
            timeframe,
            side,
            min_usdc,
            smart_money_only,
            limit,
        } => {
            let condition_id = resolve_condition_id(&market).await?;
            let limit_s = limit.to_string();
            let mut q = vec![
                ("condition_id", condition_id.as_str()),
                ("timeframe", timeframe.as_str()),
                ("limit", limit_s.as_str()),
            ];
            push_opt(&mut q, "side", &side);
            let mu_s = min_usdc.map(|v| v.to_string());
            push_opt(&mut q, "min_usdc", &mu_s);
            if smart_money_only {
                q.push(("smart_money_only", "true"));
            }
            let data = client.get("/api/v1/markets/trades", &q).await?;
            out::print_trades(&data, output)
        }
        MetengineCommand::WhaleTrades {
            min_usdc,
            timeframe,
            market,
            category,
            side,
            smart_money_only,
            limit,
        } => {
            let resolved_cid = match &market {
                Some(m) => Some(resolve_condition_id(m).await?),
                None => None,
            };
            let limit_s = limit.to_string();
            let mu_s = min_usdc.to_string();
            let mut q = vec![
                ("min_usdc", mu_s.as_str()),
                ("timeframe", timeframe.as_str()),
                ("limit", limit_s.as_str()),
            ];
            push_opt(&mut q, "condition_id", &resolved_cid);
            push_opt(&mut q, "category", &category);
            push_opt(&mut q, "side", &side);
            if smart_money_only {
                q.push(("smart_money_only", "true"));
            }
            let data = client.get("/api/v1/trades/whales", &q).await?;
            out::print_whale_trades(&data, output)
        }
        MetengineCommand::AlphaCallers {
            days_back,
            min_days_early,
            min_bet_usdc,
            limit,
        } => {
            let db_s = days_back.to_string();
            let mde_s = min_days_early.to_string();
            let mbu_s = min_bet_usdc.to_string();
            let limit_s = limit.to_string();
            let q = vec![
                ("days_back", db_s.as_str()),
                ("min_days_early", &mde_s),
                ("min_bet_usdc", &mbu_s),
                ("limit", &limit_s),
            ];
            let data = client.get("/api/v1/wallets/alpha-callers", &q).await?;
            out::print_alpha_callers(&data, output)
        }
        MetengineCommand::WalletInsiders {
            limit,
            min_score,
            max_wallet_age_days,
        } => {
            let limit_s = limit.to_string();
            let ms_s = min_score.to_string();
            let mwad_s = max_wallet_age_days.to_string();
            let q = vec![
                ("limit", limit_s.as_str()),
                ("min_score", &ms_s),
                ("max_wallet_age_days", &mwad_s),
            ];
            let data = client.get("/api/v1/wallets/insiders", &q).await?;
            out::print_wallet_insiders(&data, output)
        }

        // -- Wallet Analytics (POST) --
        MetengineCommand::WalletProfile {
            wallet,
            include_positions,
            include_trades,
            trades_limit,
        } => {
            let body = json!({
                "wallet": wallet.to_lowercase(),
                "include_positions": include_positions,
                "include_trades": include_trades,
                "trades_limit": trades_limit,
            });
            let data = client.post("/api/v1/wallets/profile", &body).await?;
            out::print_wallet_profile(&data, output)
        }
        MetengineCommand::WalletActivity {
            wallet,
            timeframe,
            category,
            min_usdc,
            limit,
        } => {
            let mut body = json!({
                "wallet": wallet.to_lowercase(),
                "timeframe": timeframe,
                "limit": limit,
            });
            if let Some(c) = &category {
                body["category"] = json!(c);
            }
            if let Some(mu) = min_usdc {
                body["min_usdc"] = json!(mu);
            }
            let data = client.post("/api/v1/wallets/activity", &body).await?;
            out::print_wallet_activity(&data, output)
        }
        MetengineCommand::WalletPnl {
            wallet,
            timeframe,
            limit,
        } => {
            let body = json!({
                "wallet": wallet.to_lowercase(),
                "timeframe": timeframe,
                "limit": limit,
            });
            let data = client.post("/api/v1/wallets/pnl-breakdown", &body).await?;
            out::print_wallet_pnl(&data, output)
        }
        MetengineCommand::WalletCompare {
            wallets,
            include_shared_positions,
        } => {
            let wallet_list: Vec<String> = wallets
                .split(',')
                .map(|w| w.trim().to_lowercase())
                .collect();
            if wallet_list.len() < 2 || wallet_list.len() > 5 {
                bail!("Provide 2-5 comma-separated wallet addresses");
            }
            let body = json!({
                "wallets": wallet_list,
                "include_shared_positions": include_shared_positions,
            });
            let data = client.post("/api/v1/wallets/compare", &body).await?;
            out::print_wallet_compare(&data, output)
        }
        MetengineCommand::CopyTraders {
            wallet,
            max_lag_minutes,
            timeframe,
            min_overlap_trades,
            limit,
        } => {
            let body = json!({
                "wallet": wallet.to_lowercase(),
                "max_lag_minutes": max_lag_minutes,
                "timeframe": timeframe,
                "min_overlap_trades": min_overlap_trades,
                "limit": limit,
            });
            let data = client.post("/api/v1/wallets/copy-traders", &body).await?;
            out::print_copy_traders(&data, output)
        }
        MetengineCommand::TopPerformers {
            timeframe,
            category,
            metric,
            min_trades,
            limit,
        } => {
            let limit_s = limit.to_string();
            let mt_s = min_trades.to_string();
            let mut q = vec![
                ("timeframe", timeframe.as_str()),
                ("metric", metric.as_str()),
                ("min_trades", mt_s.as_str()),
                ("limit", limit_s.as_str()),
            ];
            push_opt(&mut q, "category", &category);
            let data = client.get("/api/v1/wallets/top-performers", &q).await?;
            out::print_top_performers(&data, output)
        }
        MetengineCommand::NicheExperts {
            category,
            min_category_trades,
            sort_by,
            limit,
        } => {
            let limit_s = limit.to_string();
            let mct_s = min_category_trades.to_string();
            let q = vec![
                ("category", category.as_str()),
                ("min_category_trades", &mct_s),
                ("sort_by", sort_by.as_str()),
                ("limit", &limit_s),
            ];
            let data = client.get("/api/v1/wallets/niche-experts", &q).await?;
            out::print_niche_experts(&data, output)
        }

        MetengineCommand::Health | MetengineCommand::Pricing => unreachable!(),
    }
}
