use std::str::FromStr;

use anyhow::Result;
use chrono::NaiveDate;
use clap::{Args, Subcommand};
use polymarket_client_sdk::clob;
use polymarket_client_sdk::clob::types::{
    Amount, AssetType, Interval, OrderType, Side, TimeRange,
    request::{
        BalanceAllowanceRequest, CancelMarketOrderRequest, DeleteNotificationsRequest,
        LastTradePriceRequest, MidpointRequest, OrderBookSummaryRequest, OrdersRequest,
        PriceHistoryRequest, PriceRequest, SpreadRequest, TradesRequest, UserRewardsEarningRequest,
    },
};
use polymarket_client_sdk::types::{Decimal, U256};

use super::parse_condition_id;
use crate::auth;
use crate::output::OutputFormat;
use crate::output::clob::{
    print_account_status, print_api_keys, print_balance, print_batch_prices, print_cancel_result,
    print_clob_market, print_clob_markets, print_create_api_key, print_current_rewards,
    print_delete_api_key, print_earnings, print_fee_rate, print_geoblock, print_last_trade,
    print_last_trades_prices, print_market_reward, print_midpoint, print_midpoints, print_neg_risk,
    print_notifications, print_ok, print_order_book, print_order_books, print_order_detail,
    print_order_scoring, print_orders, print_orders_scoring, print_post_order_result,
    print_post_orders_result, print_price, print_price_history, print_reward_percentages,
    print_rewards, print_server_time, print_simplified_markets, print_spread, print_spreads,
    print_tick_size, print_trades, print_user_earnings_markets,
};

#[derive(Args)]
pub struct ClobArgs {
    #[command(subcommand)]
    pub command: ClobCommand,
}

#[derive(Subcommand)]
pub enum ClobCommand {
    // --- Unauthenticated ---
    /// Check CLOB API health
    Ok,

    /// Get price for a token
    Price {
        /// Token ID (numeric string)
        token_id: String,
        /// Side: buy or sell
        #[arg(long)]
        side: CliSide,
    },

    /// Get prices for specific tokens (batch)
    BatchPrices {
        /// Token IDs (comma-separated numeric strings)
        token_ids: String,
        /// Side: buy or sell
        #[arg(long)]
        side: CliSide,
    },

    /// Get midpoint price for a token
    Midpoint {
        /// Token ID (numeric string)
        token_id: String,
    },

    /// Get midpoints for multiple tokens
    Midpoints {
        /// Token IDs (comma-separated numeric strings)
        token_ids: String,
    },

    /// Get bid-ask spread for a token
    Spread {
        /// Token ID (numeric string)
        token_id: String,
        /// Optional side filter
        #[arg(long)]
        side: Option<CliSide>,
    },

    /// Get spreads for multiple tokens
    Spreads {
        /// Token IDs (comma-separated numeric strings)
        token_ids: String,
    },

    /// Get order book for a token
    Book {
        /// Token ID (numeric string)
        token_id: String,
    },

    /// Get order books for multiple tokens
    Books {
        /// Token IDs (comma-separated numeric strings)
        token_ids: String,
    },

    /// Get last trade price for a token
    LastTrade {
        /// Token ID (numeric string)
        token_id: String,
    },

    /// Get last trade prices for multiple tokens
    LastTrades {
        /// Token IDs (comma-separated numeric strings)
        token_ids: String,
    },

    /// Get CLOB market info by condition ID
    Market {
        /// Condition ID (0x-prefixed hex)
        condition_id: String,
    },

    /// List CLOB markets
    Markets {
        /// Pagination cursor
        #[arg(long)]
        cursor: Option<String>,
    },

    /// List sampling markets (reward-eligible)
    SamplingMarkets {
        /// Pagination cursor
        #[arg(long)]
        cursor: Option<String>,
    },

    /// List simplified markets (reduced detail)
    SimplifiedMarkets {
        /// Pagination cursor
        #[arg(long)]
        cursor: Option<String>,
    },

    /// List simplified sampling markets
    SamplingSimpMarkets {
        /// Pagination cursor
        #[arg(long)]
        cursor: Option<String>,
    },

    /// Get tick size for a token
    TickSize {
        /// Token ID (numeric string)
        token_id: String,
    },

    /// Get fee rate for a token
    FeeRate {
        /// Token ID (numeric string)
        token_id: String,
    },

    /// Check neg-risk status for a token
    NegRisk {
        /// Token ID (numeric string)
        token_id: String,
    },

    /// Get price history for a token
    PriceHistory {
        /// Token ID (numeric string)
        token_id: String,
        /// Time interval: 1m, 1h, 6h, 1d, 1w, max
        #[arg(long)]
        interval: CliInterval,
        /// Number of data points
        #[arg(long)]
        fidelity: Option<u32>,
    },

    /// Get CLOB server time
    Time,

    /// Check geoblock status
    Geoblock,

    // --- Authenticated ---
    /// List open orders (authenticated)
    Orders {
        /// Filter by market condition ID
        #[arg(long)]
        market: Option<String>,
        /// Filter by asset/token ID
        #[arg(long)]
        asset: Option<String>,
        /// Pagination cursor
        #[arg(long)]
        cursor: Option<String>,
    },

    /// Get a single order by ID (authenticated)
    Order {
        /// Order ID
        order_id: String,
    },

    /// Create a limit order (authenticated)
    CreateOrder {
        /// Token ID (numeric string)
        #[arg(long)]
        token: String,
        /// Side: buy or sell
        #[arg(long)]
        side: CliSide,
        /// Price (decimal, e.g. 0.50)
        #[arg(long)]
        price: String,
        /// Size (number of shares, e.g. 10)
        #[arg(long)]
        size: String,
        /// Order type: GTC, FOK, GTD, FAK (default: GTC)
        #[arg(long, default_value = "GTC")]
        order_type: CliOrderType,
        /// Post-only order
        #[arg(long)]
        post_only: bool,
    },

    /// Post multiple orders at once (authenticated)
    PostOrders {
        /// Token IDs (comma-separated, one per order)
        #[arg(long)]
        tokens: String,
        /// Side: buy or sell (same for all)
        #[arg(long)]
        side: CliSide,
        /// Prices (comma-separated, one per order)
        #[arg(long)]
        prices: String,
        /// Sizes (comma-separated, one per order)
        #[arg(long)]
        sizes: String,
        /// Order type: GTC, FOK, GTD, FAK (default: GTC)
        #[arg(long, default_value = "GTC")]
        order_type: CliOrderType,
    },

    /// Create a market order (authenticated)
    MarketOrder {
        /// Token ID (numeric string)
        #[arg(long)]
        token: String,
        /// Side: buy or sell
        #[arg(long)]
        side: CliSide,
        /// Amount (USDC for buys, shares for sells)
        #[arg(long)]
        amount: String,
        /// Order type: FOK or FAK (default: FOK)
        #[arg(long, default_value = "FOK")]
        order_type: CliOrderType,
    },

    /// Cancel an order by ID (authenticated)
    Cancel {
        /// Order ID to cancel
        order_id: String,
    },

    /// Cancel multiple orders by IDs (authenticated)
    CancelOrders {
        /// Order IDs (comma-separated)
        order_ids: String,
    },

    /// Cancel all open orders (authenticated)
    CancelAll,

    /// Cancel orders for a specific market (authenticated)
    CancelMarket {
        /// Market condition ID
        #[arg(long)]
        market: Option<String>,
        /// Asset/token ID
        #[arg(long)]
        asset: Option<String>,
    },

    /// List trades (authenticated)
    Trades {
        /// Filter by market condition ID
        #[arg(long)]
        market: Option<String>,
        /// Filter by asset/token ID
        #[arg(long)]
        asset: Option<String>,
        /// Pagination cursor
        #[arg(long)]
        cursor: Option<String>,
    },

    /// Get balance and allowance (authenticated)
    Balance {
        /// Asset type: collateral or conditional
        #[arg(long)]
        asset_type: CliAssetType,
        /// Token ID (required for conditional)
        #[arg(long)]
        token: Option<String>,
    },

    /// Refresh balance allowance on-chain (authenticated)
    UpdateBalance {
        /// Asset type: collateral or conditional
        #[arg(long)]
        asset_type: CliAssetType,
        /// Token ID (required for conditional)
        #[arg(long)]
        token: Option<String>,
    },

    /// List notifications (authenticated)
    Notifications,

    /// Delete notifications by IDs (authenticated)
    DeleteNotifications {
        /// Notification IDs (comma-separated)
        ids: String,
    },

    /// List reward earnings (authenticated)
    Rewards {
        /// Date (YYYY-MM-DD)
        #[arg(long)]
        date: String,
        /// Pagination cursor
        #[arg(long)]
        cursor: Option<String>,
    },

    /// Get total earnings for a date (authenticated)
    Earnings {
        /// Date (YYYY-MM-DD)
        #[arg(long)]
        date: String,
    },

    /// Get earnings with market reward config (authenticated)
    EarningsMarkets {
        /// Date (YYYY-MM-DD)
        #[arg(long)]
        date: String,
        /// Pagination cursor
        #[arg(long)]
        cursor: Option<String>,
    },

    /// Get reward percentages (authenticated)
    RewardPercentages,

    /// List current reward programs (authenticated)
    CurrentRewards {
        /// Pagination cursor
        #[arg(long)]
        cursor: Option<String>,
    },

    /// Get reward details for a market (authenticated)
    MarketReward {
        /// Market condition ID
        condition_id: String,
        /// Pagination cursor
        #[arg(long)]
        cursor: Option<String>,
    },

    /// Check if an order is scoring rewards (authenticated)
    OrderScoring {
        /// Order ID
        order_id: String,
    },

    /// Check if multiple orders are scoring rewards (authenticated)
    OrdersScoring {
        /// Order IDs (comma-separated)
        order_ids: String,
    },

    /// List API keys (authenticated)
    ApiKeys,

    /// Delete current API key (authenticated)
    DeleteApiKey,

    /// Create or derive an API key (authenticated)
    CreateApiKey,

    /// Check account status (authenticated)
    AccountStatus,
}

#[derive(Clone, Debug, clap::ValueEnum)]
pub enum CliSide {
    Buy,
    Sell,
}

impl From<CliSide> for Side {
    fn from(s: CliSide) -> Self {
        match s {
            CliSide::Buy => Side::Buy,
            CliSide::Sell => Side::Sell,
        }
    }
}

#[derive(Clone, Debug, clap::ValueEnum)]
pub enum CliInterval {
    #[value(name = "1m")]
    OneMinute,
    #[value(name = "1h")]
    OneHour,
    #[value(name = "6h")]
    SixHours,
    #[value(name = "1d")]
    OneDay,
    #[value(name = "1w")]
    OneWeek,
    Max,
}

impl From<CliInterval> for Interval {
    fn from(i: CliInterval) -> Self {
        match i {
            CliInterval::OneMinute => Interval::OneMinute,
            CliInterval::OneHour => Interval::OneHour,
            CliInterval::SixHours => Interval::SixHours,
            CliInterval::OneDay => Interval::OneDay,
            CliInterval::OneWeek => Interval::OneWeek,
            CliInterval::Max => Interval::Max,
        }
    }
}

#[derive(Clone, Debug, clap::ValueEnum)]
pub enum CliOrderType {
    #[value(name = "GTC")]
    Gtc,
    #[value(name = "FOK")]
    Fok,
    #[value(name = "GTD")]
    Gtd,
    #[value(name = "FAK")]
    Fak,
}

impl From<CliOrderType> for OrderType {
    fn from(o: CliOrderType) -> Self {
        match o {
            CliOrderType::Gtc => OrderType::GTC,
            CliOrderType::Fok => OrderType::FOK,
            CliOrderType::Gtd => OrderType::GTD,
            CliOrderType::Fak => OrderType::FAK,
        }
    }
}

#[derive(Clone, Debug, clap::ValueEnum)]
pub enum CliAssetType {
    Collateral,
    Conditional,
}

impl From<CliAssetType> for AssetType {
    fn from(a: CliAssetType) -> Self {
        match a {
            CliAssetType::Collateral => AssetType::Collateral,
            CliAssetType::Conditional => AssetType::Conditional,
        }
    }
}

fn parse_token_id(s: &str) -> Result<U256> {
    U256::from_str(s).map_err(|_| anyhow::anyhow!("Invalid token ID: {s}"))
}

fn parse_token_ids(s: &str) -> Result<Vec<U256>> {
    s.split(',').map(|t| parse_token_id(t.trim())).collect()
}

fn parse_date(s: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .map_err(|_| anyhow::anyhow!("Invalid date: expected YYYY-MM-DD format"))
}

#[allow(clippy::too_many_lines)]
pub async fn execute(
    args: ClobArgs,
    output: OutputFormat,
    private_key: Option<&str>,
    signature_type: Option<&str>,
) -> Result<()> {
    match args.command {
        // --- Unauthenticated commands ---
        ClobCommand::Ok => {
            let client = clob::Client::default();
            let result = client.ok().await?;
            print_ok(&result, &output);
        }

        ClobCommand::Price { token_id, side } => {
            let client = clob::Client::default();
            let request = PriceRequest::builder()
                .token_id(parse_token_id(&token_id)?)
                .side(Side::from(side))
                .build();
            let result = client.price(&request).await?;
            print_price(&result, &output);
        }

        ClobCommand::BatchPrices { token_ids, side } => {
            let client = clob::Client::default();
            let requests: Vec<_> = parse_token_ids(&token_ids)?
                .into_iter()
                .map(|id| {
                    PriceRequest::builder()
                        .token_id(id)
                        .side(Side::from(side.clone()))
                        .build()
                })
                .collect();
            let result = client.prices(&requests).await?;
            print_batch_prices(&result, &output);
        }

        ClobCommand::Midpoint { token_id } => {
            let client = clob::Client::default();
            let request = MidpointRequest::builder()
                .token_id(parse_token_id(&token_id)?)
                .build();
            let result = client.midpoint(&request).await?;
            print_midpoint(&result, &output);
        }

        ClobCommand::Midpoints { token_ids } => {
            let client = clob::Client::default();
            let requests: Vec<_> = parse_token_ids(&token_ids)?
                .into_iter()
                .map(|id| MidpointRequest::builder().token_id(id).build())
                .collect();
            let result = client.midpoints(&requests).await?;
            print_midpoints(&result, &output);
        }

        ClobCommand::Spread { token_id, side } => {
            let client = clob::Client::default();
            let request = SpreadRequest::builder()
                .token_id(parse_token_id(&token_id)?)
                .maybe_side(side.map(Side::from))
                .build();
            let result = client.spread(&request).await?;
            print_spread(&result, &output);
        }

        ClobCommand::Spreads { token_ids } => {
            let client = clob::Client::default();
            let requests: Vec<_> = parse_token_ids(&token_ids)?
                .into_iter()
                .map(|id| SpreadRequest::builder().token_id(id).build())
                .collect();
            let result = client.spreads(&requests).await?;
            print_spreads(&result, &output);
        }

        ClobCommand::Book { token_id } => {
            let client = clob::Client::default();
            let request = OrderBookSummaryRequest::builder()
                .token_id(parse_token_id(&token_id)?)
                .build();
            let result = client.order_book(&request).await?;
            print_order_book(&result, &output);
        }

        ClobCommand::Books { token_ids } => {
            let client = clob::Client::default();
            let requests: Vec<_> = parse_token_ids(&token_ids)?
                .into_iter()
                .map(|id| OrderBookSummaryRequest::builder().token_id(id).build())
                .collect();
            let result = client.order_books(&requests).await?;
            print_order_books(&result, &output);
        }

        ClobCommand::LastTrade { token_id } => {
            let client = clob::Client::default();
            let request = LastTradePriceRequest::builder()
                .token_id(parse_token_id(&token_id)?)
                .build();
            let result = client.last_trade_price(&request).await?;
            print_last_trade(&result, &output);
        }

        ClobCommand::LastTrades { token_ids } => {
            let client = clob::Client::default();
            let requests: Vec<_> = parse_token_ids(&token_ids)?
                .into_iter()
                .map(|id| LastTradePriceRequest::builder().token_id(id).build())
                .collect();
            let result = client.last_trades_prices(&requests).await?;
            print_last_trades_prices(&result, &output);
        }

        ClobCommand::Market { condition_id } => {
            let client = clob::Client::default();
            let result = client.market(&condition_id).await?;
            print_clob_market(&result, &output);
        }

        ClobCommand::Markets { cursor } => {
            let client = clob::Client::default();
            let result = client.markets(cursor).await?;
            print_clob_markets(&result, &output);
        }

        ClobCommand::SamplingMarkets { cursor } => {
            let client = clob::Client::default();
            let result = client.sampling_markets(cursor).await?;
            print_clob_markets(&result, &output);
        }

        ClobCommand::SimplifiedMarkets { cursor } => {
            let client = clob::Client::default();
            let result = client.simplified_markets(cursor).await?;
            print_simplified_markets(&result, &output);
        }

        ClobCommand::SamplingSimpMarkets { cursor } => {
            let client = clob::Client::default();
            let result = client.sampling_simplified_markets(cursor).await?;
            print_simplified_markets(&result, &output);
        }

        ClobCommand::TickSize { token_id } => {
            let client = clob::Client::default();
            let result = client.tick_size(parse_token_id(&token_id)?).await?;
            print_tick_size(&result, &output);
        }

        ClobCommand::FeeRate { token_id } => {
            let client = clob::Client::default();
            let result = client.fee_rate_bps(parse_token_id(&token_id)?).await?;
            print_fee_rate(&result, &output);
        }

        ClobCommand::NegRisk { token_id } => {
            let client = clob::Client::default();
            let result = client.neg_risk(parse_token_id(&token_id)?).await?;
            print_neg_risk(&result, &output);
        }

        ClobCommand::PriceHistory {
            token_id,
            interval,
            fidelity,
        } => {
            let client = clob::Client::default();
            let request = PriceHistoryRequest::builder()
                .market(parse_token_id(&token_id)?)
                .time_range(TimeRange::from_interval(Interval::from(interval)))
                .maybe_fidelity(fidelity)
                .build();
            let result = client.price_history(&request).await?;
            print_price_history(&result, &output);
        }

        ClobCommand::Time => {
            let client = clob::Client::default();
            let result = client.server_time().await?;
            print_server_time(result, &output);
        }

        ClobCommand::Geoblock => {
            let client = clob::Client::default();
            let result = client.check_geoblock().await?;
            print_geoblock(&result, &output);
        }

        // --- Authenticated commands ---
        ClobCommand::Orders {
            market,
            asset,
            cursor,
        } => {
            let client = auth::authenticated_clob_client(private_key, signature_type).await?;
            let request = OrdersRequest::builder()
                .maybe_market(market.map(|m| parse_condition_id(&m)).transpose()?)
                .maybe_asset_id(asset.map(|a| parse_token_id(&a)).transpose()?)
                .build();
            let result = client.orders(&request, cursor).await?;
            print_orders(&result, &output);
        }

        ClobCommand::Order { order_id } => {
            let client = auth::authenticated_clob_client(private_key, signature_type).await?;
            let result = client.order(&order_id).await?;
            print_order_detail(&result, &output);
        }

        ClobCommand::CreateOrder {
            token,
            side,
            price,
            size,
            order_type,
            post_only,
        } => {
            let signer = auth::resolve_signer(private_key)?;
            let client = auth::authenticated_clob_client(private_key, signature_type).await?;

            let price_dec =
                Decimal::from_str(&price).map_err(|_| anyhow::anyhow!("Invalid price: {price}"))?;
            let size_dec =
                Decimal::from_str(&size).map_err(|_| anyhow::anyhow!("Invalid size: {size}"))?;

            let order = client
                .limit_order()
                .token_id(parse_token_id(&token)?)
                .side(Side::from(side))
                .price(price_dec)
                .size(size_dec)
                .order_type(OrderType::from(order_type))
                .post_only(post_only)
                .build()
                .await?;
            let order = client.sign(&signer, order).await?;
            let result = client.post_order(order).await?;
            print_post_order_result(&result, &output);
        }

        ClobCommand::PostOrders {
            tokens,
            side,
            prices,
            sizes,
            order_type,
        } => {
            let signer = auth::resolve_signer(private_key)?;
            let client = auth::authenticated_clob_client(private_key, signature_type).await?;

            let token_ids = parse_token_ids(&tokens)?;
            let price_strs: Vec<&str> = prices.split(',').map(str::trim).collect();
            let size_strs: Vec<&str> = sizes.split(',').map(str::trim).collect();

            if token_ids.len() != price_strs.len() || token_ids.len() != size_strs.len() {
                anyhow::bail!(
                    "tokens, prices, and sizes must have the same number of comma-separated values"
                );
            }

            let sdk_side = Side::from(side);
            let sdk_order_type = OrderType::from(order_type);

            let mut signed_orders = Vec::with_capacity(token_ids.len());
            for ((token_id, price_str), size_str) in
                token_ids.into_iter().zip(price_strs).zip(size_strs)
            {
                let price_dec = Decimal::from_str(price_str)
                    .map_err(|_| anyhow::anyhow!("Invalid price: {price_str}"))?;
                let size_dec = Decimal::from_str(size_str)
                    .map_err(|_| anyhow::anyhow!("Invalid size: {size_str}"))?;

                let order = client
                    .limit_order()
                    .token_id(token_id)
                    .side(sdk_side)
                    .price(price_dec)
                    .size(size_dec)
                    .order_type(sdk_order_type.clone())
                    .build()
                    .await?;
                signed_orders.push(client.sign(&signer, order).await?);
            }

            let results = client.post_orders(signed_orders).await?;
            print_post_orders_result(&results, &output);
        }

        ClobCommand::MarketOrder {
            token,
            side,
            amount,
            order_type,
        } => {
            let signer = auth::resolve_signer(private_key)?;
            let client = auth::authenticated_clob_client(private_key, signature_type).await?;

            let amount_dec = Decimal::from_str(&amount)
                .map_err(|_| anyhow::anyhow!("Invalid amount: {amount}"))?;
            let sdk_side = Side::from(side);
            let parsed_amount = if matches!(sdk_side, Side::Sell) {
                Amount::shares(amount_dec)?
            } else {
                Amount::usdc(amount_dec)?
            };

            let order = client
                .market_order()
                .token_id(parse_token_id(&token)?)
                .side(sdk_side)
                .amount(parsed_amount)
                .order_type(OrderType::from(order_type))
                .build()
                .await?;
            let order = client.sign(&signer, order).await?;
            let result = client.post_order(order).await?;
            print_post_order_result(&result, &output);
        }

        ClobCommand::Cancel { order_id } => {
            let client = auth::authenticated_clob_client(private_key, signature_type).await?;
            let result = client.cancel_order(&order_id).await?;
            print_cancel_result(&result, &output);
        }

        ClobCommand::CancelOrders { order_ids } => {
            let client = auth::authenticated_clob_client(private_key, signature_type).await?;
            let ids: Vec<&str> = order_ids.split(',').map(str::trim).collect();
            let result = client.cancel_orders(&ids).await?;
            print_cancel_result(&result, &output);
        }

        ClobCommand::CancelAll => {
            let client = auth::authenticated_clob_client(private_key, signature_type).await?;
            let result = client.cancel_all_orders().await?;
            print_cancel_result(&result, &output);
        }

        ClobCommand::CancelMarket { market, asset } => {
            let client = auth::authenticated_clob_client(private_key, signature_type).await?;
            let request = CancelMarketOrderRequest::builder()
                .maybe_market(market.map(|m| parse_condition_id(&m)).transpose()?)
                .maybe_asset_id(asset.map(|a| parse_token_id(&a)).transpose()?)
                .build();
            let result = client.cancel_market_orders(&request).await?;
            print_cancel_result(&result, &output);
        }

        ClobCommand::Trades {
            market,
            asset,
            cursor,
        } => {
            let client = auth::authenticated_clob_client(private_key, signature_type).await?;
            let request = TradesRequest::builder()
                .maybe_market(market.map(|m| parse_condition_id(&m)).transpose()?)
                .maybe_asset_id(asset.map(|a| parse_token_id(&a)).transpose()?)
                .build();
            let result = client.trades(&request, cursor).await?;
            print_trades(&result, &output);
        }

        ClobCommand::Balance { asset_type, token } => {
            let is_collateral = matches!(asset_type, CliAssetType::Collateral);
            let client = auth::authenticated_clob_client(private_key, signature_type).await?;
            let request = BalanceAllowanceRequest::builder()
                .asset_type(AssetType::from(asset_type))
                .maybe_token_id(token.map(|t| parse_token_id(&t)).transpose()?)
                .build();
            let result = client.balance_allowance(request).await?;
            print_balance(&result, is_collateral, &output);
        }

        ClobCommand::UpdateBalance { asset_type, token } => {
            let client = auth::authenticated_clob_client(private_key, signature_type).await?;
            let request = BalanceAllowanceRequest::builder()
                .asset_type(AssetType::from(asset_type))
                .maybe_token_id(token.map(|t| parse_token_id(&t)).transpose()?)
                .build();
            client.update_balance_allowance(request).await?;
            match output {
                OutputFormat::Table => println!("Balance allowance updated."),
                OutputFormat::Json => {
                    println!("{}", serde_json::json!({"success": true}));
                }
            }
        }

        ClobCommand::Notifications => {
            let client = auth::authenticated_clob_client(private_key, signature_type).await?;
            let result = client.notifications().await?;
            print_notifications(&result, &output);
        }

        ClobCommand::DeleteNotifications { ids } => {
            let client = auth::authenticated_clob_client(private_key, signature_type).await?;
            let notification_ids: Vec<String> =
                ids.split(',').map(|s| s.trim().to_string()).collect();
            let request = DeleteNotificationsRequest::builder()
                .notification_ids(notification_ids)
                .build();
            client.delete_notifications(&request).await?;
            match output {
                OutputFormat::Table => println!("Notifications deleted."),
                OutputFormat::Json => {
                    println!("{}", serde_json::json!({"success": true}));
                }
            }
        }

        ClobCommand::Rewards { date, cursor } => {
            let client = auth::authenticated_clob_client(private_key, signature_type).await?;
            let result = client
                .earnings_for_user_for_day(parse_date(&date)?, cursor)
                .await?;
            print_rewards(&result, &output);
        }

        ClobCommand::Earnings { date } => {
            let client = auth::authenticated_clob_client(private_key, signature_type).await?;
            let result = client
                .total_earnings_for_user_for_day(parse_date(&date)?)
                .await?;
            print_earnings(&result, &output);
        }

        ClobCommand::EarningsMarkets { date, cursor } => {
            let client = auth::authenticated_clob_client(private_key, signature_type).await?;
            let request = UserRewardsEarningRequest::builder()
                .date(parse_date(&date)?)
                .build();
            let result = client
                .user_earnings_and_markets_config(&request, cursor)
                .await?;
            print_user_earnings_markets(&result, &output);
        }

        ClobCommand::RewardPercentages => {
            let client = auth::authenticated_clob_client(private_key, signature_type).await?;
            let result = client.reward_percentages().await?;
            print_reward_percentages(&result, &output);
        }

        ClobCommand::CurrentRewards { cursor } => {
            let client = auth::authenticated_clob_client(private_key, signature_type).await?;
            let result = client.current_rewards(cursor).await?;
            print_current_rewards(&result, &output);
        }

        ClobCommand::MarketReward {
            condition_id,
            cursor,
        } => {
            let client = auth::authenticated_clob_client(private_key, signature_type).await?;
            let result = client.raw_rewards_for_market(&condition_id, cursor).await?;
            print_market_reward(&result, &output);
        }

        ClobCommand::OrderScoring { order_id } => {
            let client = auth::authenticated_clob_client(private_key, signature_type).await?;
            let result = client.is_order_scoring(&order_id).await?;
            print_order_scoring(&result, &output);
        }

        ClobCommand::OrdersScoring { order_ids } => {
            let client = auth::authenticated_clob_client(private_key, signature_type).await?;
            let ids: Vec<&str> = order_ids.split(',').map(str::trim).collect();
            let result = client.are_orders_scoring(&ids).await?;
            print_orders_scoring(&result, &output);
        }

        ClobCommand::ApiKeys => {
            let client = auth::authenticated_clob_client(private_key, signature_type).await?;
            let result = client.api_keys().await?;
            print_api_keys(&result, &output);
        }

        ClobCommand::DeleteApiKey => {
            let client = auth::authenticated_clob_client(private_key, signature_type).await?;
            let result = client.delete_api_key().await?;
            print_delete_api_key(&result, &output);
        }

        ClobCommand::CreateApiKey => {
            let signer = auth::resolve_signer(private_key)?;
            let client = clob::Client::default();
            let result = client.create_or_derive_api_key(&signer, None).await?;
            print_create_api_key(&result, &output);
        }

        ClobCommand::AccountStatus => {
            let client = auth::authenticated_clob_client(private_key, signature_type).await?;
            let result = client.closed_only_mode().await?;
            print_account_status(&result, &output);
        }
    }

    Ok(())
}
