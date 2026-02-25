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
use polymarket_client_sdk::types::{B256, Decimal, U256};
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

    /// List open orders (authenticated)
    Orders {
        /// Filter by market condition ID
        #[arg(long)]
        market: Option<B256>,
        /// Filter by asset/token ID
        #[arg(long)]
        asset: Option<U256>,
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
        market: Option<B256>,
        /// Asset/token ID
        #[arg(long)]
        asset: Option<U256>,
    },

    /// List trades (authenticated)
    Trades {
        /// Filter by market condition ID
        #[arg(long)]
        market: Option<B256>,
        /// Filter by asset/token ID
        #[arg(long)]
        asset: Option<U256>,
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
        token: Option<U256>,
    },

    /// Refresh balance allowance on-chain (authenticated)
    UpdateBalance {
        /// Asset type: collateral or conditional
        #[arg(long)]
        asset_type: CliAssetType,
        /// Token ID (required for conditional)
        #[arg(long)]
        token: Option<U256>,
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

#[derive(Clone, Copy, Debug, clap::ValueEnum)]
pub enum CliSide {
    Buy,
    Sell,
}

super::enum_from!(CliSide => Side { Buy, Sell });

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

super::enum_from!(CliInterval => Interval { OneMinute, OneHour, SixHours, OneDay, OneWeek, Max });

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

super::enum_from!(CliAssetType => AssetType { Collateral, Conditional });

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

pub async fn execute(
    args: ClobArgs,
    output: OutputFormat,
    private_key: Option<&str>,
    signature_type: Option<&str>,
) -> Result<()> {
    match args.command {
        // Unauthenticated read commands
        ClobCommand::Ok
        | ClobCommand::Price { .. }
        | ClobCommand::BatchPrices { .. }
        | ClobCommand::Midpoint { .. }
        | ClobCommand::Midpoints { .. }
        | ClobCommand::Spread { .. }
        | ClobCommand::Spreads { .. }
        | ClobCommand::Book { .. }
        | ClobCommand::Books { .. }
        | ClobCommand::LastTrade { .. }
        | ClobCommand::LastTrades { .. }
        | ClobCommand::Market { .. }
        | ClobCommand::Markets { .. }
        | ClobCommand::SamplingMarkets { .. }
        | ClobCommand::SimplifiedMarkets { .. }
        | ClobCommand::SamplingSimpMarkets { .. }
        | ClobCommand::TickSize { .. }
        | ClobCommand::FeeRate { .. }
        | ClobCommand::NegRisk { .. }
        | ClobCommand::PriceHistory { .. }
        | ClobCommand::Time
        | ClobCommand::Geoblock => execute_read(args.command, &output).await,

        // Authenticated trading commands
        ClobCommand::Orders { .. }
        | ClobCommand::Order { .. }
        | ClobCommand::CreateOrder { .. }
        | ClobCommand::PostOrders { .. }
        | ClobCommand::MarketOrder { .. }
        | ClobCommand::Cancel { .. }
        | ClobCommand::CancelOrders { .. }
        | ClobCommand::CancelAll
        | ClobCommand::CancelMarket { .. }
        | ClobCommand::Trades { .. }
        | ClobCommand::Balance { .. }
        | ClobCommand::UpdateBalance { .. }
        | ClobCommand::Notifications
        | ClobCommand::DeleteNotifications { .. } => {
            execute_trade(args.command, &output, private_key, signature_type).await
        }

        // Authenticated reward commands
        ClobCommand::Rewards { .. }
        | ClobCommand::Earnings { .. }
        | ClobCommand::EarningsMarkets { .. }
        | ClobCommand::RewardPercentages
        | ClobCommand::CurrentRewards { .. }
        | ClobCommand::MarketReward { .. }
        | ClobCommand::OrderScoring { .. }
        | ClobCommand::OrdersScoring { .. } => {
            execute_rewards(args.command, &output, private_key, signature_type).await
        }

        // Account management commands
        ClobCommand::ApiKeys
        | ClobCommand::DeleteApiKey
        | ClobCommand::CreateApiKey
        | ClobCommand::AccountStatus => {
            execute_account(args.command, &output, private_key, signature_type).await
        }
    }
}

async fn execute_read(command: ClobCommand, output: &OutputFormat) -> Result<()> {
    let client = clob::Client::default();

    match command {
        ClobCommand::Ok => {
            let result = client.ok().await?;
            print_ok(&result, output)?;
        }

        ClobCommand::Price { token_id, side } => {
            let request = PriceRequest::builder()
                .token_id(parse_token_id(&token_id)?)
                .side(Side::from(side))
                .build();
            let result = client.price(&request).await?;
            print_price(&result, output)?;
        }

        ClobCommand::BatchPrices { token_ids, side } => {
            let requests: Vec<_> = parse_token_ids(&token_ids)?
                .into_iter()
                .map(|id| {
                    PriceRequest::builder()
                        .token_id(id)
                        .side(Side::from(side))
                        .build()
                })
                .collect();
            let result = client.prices(&requests).await?;
            print_batch_prices(&result, output)?;
        }

        ClobCommand::Midpoint { token_id } => {
            let request = MidpointRequest::builder()
                .token_id(parse_token_id(&token_id)?)
                .build();
            let result = client.midpoint(&request).await?;
            print_midpoint(&result, output)?;
        }

        ClobCommand::Midpoints { token_ids } => {
            let requests: Vec<_> = parse_token_ids(&token_ids)?
                .into_iter()
                .map(|id| MidpointRequest::builder().token_id(id).build())
                .collect();
            let result = client.midpoints(&requests).await?;
            print_midpoints(&result, output)?;
        }

        ClobCommand::Spread { token_id, side } => {
            let request = SpreadRequest::builder()
                .token_id(parse_token_id(&token_id)?)
                .maybe_side(side.map(Side::from))
                .build();
            let result = client.spread(&request).await?;
            print_spread(&result, output)?;
        }

        ClobCommand::Spreads { token_ids } => {
            let requests: Vec<_> = parse_token_ids(&token_ids)?
                .into_iter()
                .map(|id| SpreadRequest::builder().token_id(id).build())
                .collect();
            let result = client.spreads(&requests).await?;
            print_spreads(&result, output)?;
        }

        ClobCommand::Book { token_id } => {
            let request = OrderBookSummaryRequest::builder()
                .token_id(parse_token_id(&token_id)?)
                .build();
            let result = client.order_book(&request).await?;
            print_order_book(&result, output)?;
        }

        ClobCommand::Books { token_ids } => {
            let requests: Vec<_> = parse_token_ids(&token_ids)?
                .into_iter()
                .map(|id| OrderBookSummaryRequest::builder().token_id(id).build())
                .collect();
            let result = client.order_books(&requests).await?;
            print_order_books(&result, output)?;
        }

        ClobCommand::LastTrade { token_id } => {
            let request = LastTradePriceRequest::builder()
                .token_id(parse_token_id(&token_id)?)
                .build();
            let result = client.last_trade_price(&request).await?;
            print_last_trade(&result, output)?;
        }

        ClobCommand::LastTrades { token_ids } => {
            let requests: Vec<_> = parse_token_ids(&token_ids)?
                .into_iter()
                .map(|id| LastTradePriceRequest::builder().token_id(id).build())
                .collect();
            let result = client.last_trades_prices(&requests).await?;
            print_last_trades_prices(&result, output)?;
        }

        ClobCommand::Market { condition_id } => {
            let result = client.market(&condition_id).await?;
            print_clob_market(&result, output)?;
        }

        ClobCommand::Markets { cursor } => {
            let result = client.markets(cursor).await?;
            print_clob_markets(&result, output)?;
        }

        ClobCommand::SamplingMarkets { cursor } => {
            let result = client.sampling_markets(cursor).await?;
            print_clob_markets(&result, output)?;
        }

        ClobCommand::SimplifiedMarkets { cursor } => {
            let result = client.simplified_markets(cursor).await?;
            print_simplified_markets(&result, output)?;
        }

        ClobCommand::SamplingSimpMarkets { cursor } => {
            let result = client.sampling_simplified_markets(cursor).await?;
            print_simplified_markets(&result, output)?;
        }

        ClobCommand::TickSize { token_id } => {
            let result = client.tick_size(parse_token_id(&token_id)?).await?;
            print_tick_size(&result, output)?;
        }

        ClobCommand::FeeRate { token_id } => {
            let result = client.fee_rate_bps(parse_token_id(&token_id)?).await?;
            print_fee_rate(&result, output)?;
        }

        ClobCommand::NegRisk { token_id } => {
            let result = client.neg_risk(parse_token_id(&token_id)?).await?;
            print_neg_risk(&result, output)?;
        }

        ClobCommand::PriceHistory {
            token_id,
            interval,
            fidelity,
        } => {
            let request = PriceHistoryRequest::builder()
                .market(parse_token_id(&token_id)?)
                .time_range(TimeRange::from_interval(Interval::from(interval)))
                .maybe_fidelity(fidelity)
                .build();
            let result = client.price_history(&request).await?;
            print_price_history(&result, output)?;
        }

        ClobCommand::Time => {
            let result = client.server_time().await?;
            print_server_time(result, output)?;
        }

        ClobCommand::Geoblock => {
            let result = client.check_geoblock().await?;
            print_geoblock(&result, output)?;
        }

        // Trade, reward, and account commands are dispatched by execute() above.
        ClobCommand::Orders { .. }
        | ClobCommand::Order { .. }
        | ClobCommand::CreateOrder { .. }
        | ClobCommand::PostOrders { .. }
        | ClobCommand::MarketOrder { .. }
        | ClobCommand::Cancel { .. }
        | ClobCommand::CancelOrders { .. }
        | ClobCommand::CancelAll
        | ClobCommand::CancelMarket { .. }
        | ClobCommand::Trades { .. }
        | ClobCommand::Balance { .. }
        | ClobCommand::UpdateBalance { .. }
        | ClobCommand::Notifications
        | ClobCommand::DeleteNotifications { .. }
        | ClobCommand::Rewards { .. }
        | ClobCommand::Earnings { .. }
        | ClobCommand::EarningsMarkets { .. }
        | ClobCommand::RewardPercentages
        | ClobCommand::CurrentRewards { .. }
        | ClobCommand::MarketReward { .. }
        | ClobCommand::OrderScoring { .. }
        | ClobCommand::OrdersScoring { .. }
        | ClobCommand::ApiKeys
        | ClobCommand::DeleteApiKey
        | ClobCommand::CreateApiKey
        | ClobCommand::AccountStatus => {
            unreachable!("execute() routes authenticated commands to other handlers")
        }
    }

    Ok(())
}

async fn execute_trade(
    command: ClobCommand,
    output: &OutputFormat,
    private_key: Option<&str>,
    signature_type: Option<&str>,
) -> Result<()> {
    let signer = auth::resolve_signer(private_key)?;
    let client = auth::authenticate_with_signer(&signer, signature_type).await?;

    match command {
        ClobCommand::Orders {
            market,
            asset,
            cursor,
        } => {
            let request = OrdersRequest::builder()
                .maybe_market(market)
                .maybe_asset_id(asset)
                .build();
            let result = client.orders(&request, cursor).await?;
            print_orders(&result, output)?;
        }

        ClobCommand::Order { order_id } => {
            let result = client.order(&order_id).await?;
            print_order_detail(&result, output)?;
        }

        ClobCommand::CreateOrder {
            token,
            side,
            price,
            size,
            order_type,
            post_only,
        } => {
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
            print_post_order_result(&result, output)?;
        }

        ClobCommand::PostOrders {
            tokens,
            side,
            prices,
            sizes,
            order_type,
        } => {
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
            print_post_orders_result(&results, output)?;
        }

        ClobCommand::MarketOrder {
            token,
            side,
            amount,
            order_type,
        } => {
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
            print_post_order_result(&result, output)?;
        }

        ClobCommand::Cancel { order_id } => {
            let result = client.cancel_order(&order_id).await?;
            print_cancel_result(&result, output)?;
        }

        ClobCommand::CancelOrders { order_ids } => {
            let ids: Vec<&str> = order_ids.split(',').map(str::trim).collect();
            let result = client.cancel_orders(&ids).await?;
            print_cancel_result(&result, output)?;
        }

        ClobCommand::CancelAll => {
            let result = client.cancel_all_orders().await?;
            print_cancel_result(&result, output)?;
        }

        ClobCommand::CancelMarket { market, asset } => {
            let request = CancelMarketOrderRequest::builder()
                .maybe_market(market)
                .maybe_asset_id(asset)
                .build();
            let result = client.cancel_market_orders(&request).await?;
            print_cancel_result(&result, output)?;
        }

        ClobCommand::Trades {
            market,
            asset,
            cursor,
        } => {
            let request = TradesRequest::builder()
                .maybe_market(market)
                .maybe_asset_id(asset)
                .build();
            let result = client.trades(&request, cursor).await?;
            print_trades(&result, output)?;
        }

        ClobCommand::Balance { asset_type, token } => {
            let is_collateral = matches!(asset_type, CliAssetType::Collateral);
            let request = BalanceAllowanceRequest::builder()
                .asset_type(AssetType::from(asset_type))
                .maybe_token_id(token)
                .build();
            let result = client.balance_allowance(request).await?;
            print_balance(&result, is_collateral, output)?;
        }

        ClobCommand::UpdateBalance { asset_type, token } => {
            let request = BalanceAllowanceRequest::builder()
                .asset_type(AssetType::from(asset_type))
                .maybe_token_id(token)
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
            let result = client.notifications().await?;
            print_notifications(&result, output)?;
        }

        ClobCommand::DeleteNotifications { ids } => {
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

        // Read, reward, and account commands are dispatched by execute() above.
        ClobCommand::Ok
        | ClobCommand::Price { .. }
        | ClobCommand::BatchPrices { .. }
        | ClobCommand::Midpoint { .. }
        | ClobCommand::Midpoints { .. }
        | ClobCommand::Spread { .. }
        | ClobCommand::Spreads { .. }
        | ClobCommand::Book { .. }
        | ClobCommand::Books { .. }
        | ClobCommand::LastTrade { .. }
        | ClobCommand::LastTrades { .. }
        | ClobCommand::Market { .. }
        | ClobCommand::Markets { .. }
        | ClobCommand::SamplingMarkets { .. }
        | ClobCommand::SimplifiedMarkets { .. }
        | ClobCommand::SamplingSimpMarkets { .. }
        | ClobCommand::TickSize { .. }
        | ClobCommand::FeeRate { .. }
        | ClobCommand::NegRisk { .. }
        | ClobCommand::PriceHistory { .. }
        | ClobCommand::Time
        | ClobCommand::Geoblock
        | ClobCommand::Rewards { .. }
        | ClobCommand::Earnings { .. }
        | ClobCommand::EarningsMarkets { .. }
        | ClobCommand::RewardPercentages
        | ClobCommand::CurrentRewards { .. }
        | ClobCommand::MarketReward { .. }
        | ClobCommand::OrderScoring { .. }
        | ClobCommand::OrdersScoring { .. }
        | ClobCommand::ApiKeys
        | ClobCommand::DeleteApiKey
        | ClobCommand::CreateApiKey
        | ClobCommand::AccountStatus => {
            unreachable!("execute() routes non-trade commands to other handlers")
        }
    }

    Ok(())
}

async fn execute_rewards(
    command: ClobCommand,
    output: &OutputFormat,
    private_key: Option<&str>,
    signature_type: Option<&str>,
) -> Result<()> {
    let client = auth::authenticated_clob_client(private_key, signature_type).await?;

    match command {
        ClobCommand::Rewards { date, cursor } => {
            let result = client
                .earnings_for_user_for_day(parse_date(&date)?, cursor)
                .await?;
            print_rewards(&result, output)?;
        }

        ClobCommand::Earnings { date } => {
            let result = client
                .total_earnings_for_user_for_day(parse_date(&date)?)
                .await?;
            print_earnings(&result, output)?;
        }

        ClobCommand::EarningsMarkets { date, cursor } => {
            let request = UserRewardsEarningRequest::builder()
                .date(parse_date(&date)?)
                .build();
            let result = client
                .user_earnings_and_markets_config(&request, cursor)
                .await?;
            print_user_earnings_markets(&result, output)?;
        }

        ClobCommand::RewardPercentages => {
            let result = client.reward_percentages().await?;
            print_reward_percentages(&result, output)?;
        }

        ClobCommand::CurrentRewards { cursor } => {
            let result = client.current_rewards(cursor).await?;
            print_current_rewards(&result, output)?;
        }

        ClobCommand::MarketReward {
            condition_id,
            cursor,
        } => {
            let result = client.raw_rewards_for_market(&condition_id, cursor).await?;
            print_market_reward(&result, output)?;
        }

        ClobCommand::OrderScoring { order_id } => {
            let result = client.is_order_scoring(&order_id).await?;
            print_order_scoring(&result, output)?;
        }

        ClobCommand::OrdersScoring { order_ids } => {
            let ids: Vec<&str> = order_ids.split(',').map(str::trim).collect();
            let result = client.are_orders_scoring(&ids).await?;
            print_orders_scoring(&result, output)?;
        }

        // Read, trade, and account commands are dispatched by execute() above.
        ClobCommand::Ok
        | ClobCommand::Price { .. }
        | ClobCommand::BatchPrices { .. }
        | ClobCommand::Midpoint { .. }
        | ClobCommand::Midpoints { .. }
        | ClobCommand::Spread { .. }
        | ClobCommand::Spreads { .. }
        | ClobCommand::Book { .. }
        | ClobCommand::Books { .. }
        | ClobCommand::LastTrade { .. }
        | ClobCommand::LastTrades { .. }
        | ClobCommand::Market { .. }
        | ClobCommand::Markets { .. }
        | ClobCommand::SamplingMarkets { .. }
        | ClobCommand::SimplifiedMarkets { .. }
        | ClobCommand::SamplingSimpMarkets { .. }
        | ClobCommand::TickSize { .. }
        | ClobCommand::FeeRate { .. }
        | ClobCommand::NegRisk { .. }
        | ClobCommand::PriceHistory { .. }
        | ClobCommand::Time
        | ClobCommand::Geoblock
        | ClobCommand::Orders { .. }
        | ClobCommand::Order { .. }
        | ClobCommand::CreateOrder { .. }
        | ClobCommand::PostOrders { .. }
        | ClobCommand::MarketOrder { .. }
        | ClobCommand::Cancel { .. }
        | ClobCommand::CancelOrders { .. }
        | ClobCommand::CancelAll
        | ClobCommand::CancelMarket { .. }
        | ClobCommand::Trades { .. }
        | ClobCommand::Balance { .. }
        | ClobCommand::UpdateBalance { .. }
        | ClobCommand::Notifications
        | ClobCommand::DeleteNotifications { .. }
        | ClobCommand::ApiKeys
        | ClobCommand::DeleteApiKey
        | ClobCommand::CreateApiKey
        | ClobCommand::AccountStatus => {
            unreachable!("execute() routes non-reward commands to other handlers")
        }
    }

    Ok(())
}

async fn execute_account(
    command: ClobCommand,
    output: &OutputFormat,
    private_key: Option<&str>,
    signature_type: Option<&str>,
) -> Result<()> {
    if matches!(command, ClobCommand::CreateApiKey) {
        let signer = auth::resolve_signer(private_key)?;
        let client = clob::Client::default();
        let result = client.create_or_derive_api_key(&signer, None).await?;
        return print_create_api_key(&result, output);
    }

    let client = auth::authenticated_clob_client(private_key, signature_type).await?;

    match command {
        ClobCommand::ApiKeys => {
            let result = client.api_keys().await?;
            print_api_keys(&result, output)?;
        }

        ClobCommand::DeleteApiKey => {
            let result = client.delete_api_key().await?;
            print_delete_api_key(&result, output)?;
        }

        ClobCommand::AccountStatus => {
            let result = client.closed_only_mode().await?;
            print_account_status(&result, output)?;
        }

        // Read, trade, and reward commands are dispatched by execute() above.
        ClobCommand::Ok
        | ClobCommand::Price { .. }
        | ClobCommand::BatchPrices { .. }
        | ClobCommand::Midpoint { .. }
        | ClobCommand::Midpoints { .. }
        | ClobCommand::Spread { .. }
        | ClobCommand::Spreads { .. }
        | ClobCommand::Book { .. }
        | ClobCommand::Books { .. }
        | ClobCommand::LastTrade { .. }
        | ClobCommand::LastTrades { .. }
        | ClobCommand::Market { .. }
        | ClobCommand::Markets { .. }
        | ClobCommand::SamplingMarkets { .. }
        | ClobCommand::SimplifiedMarkets { .. }
        | ClobCommand::SamplingSimpMarkets { .. }
        | ClobCommand::TickSize { .. }
        | ClobCommand::FeeRate { .. }
        | ClobCommand::NegRisk { .. }
        | ClobCommand::PriceHistory { .. }
        | ClobCommand::Time
        | ClobCommand::Geoblock
        | ClobCommand::Orders { .. }
        | ClobCommand::Order { .. }
        | ClobCommand::CreateOrder { .. }
        | ClobCommand::PostOrders { .. }
        | ClobCommand::MarketOrder { .. }
        | ClobCommand::Cancel { .. }
        | ClobCommand::CancelOrders { .. }
        | ClobCommand::CancelAll
        | ClobCommand::CancelMarket { .. }
        | ClobCommand::Trades { .. }
        | ClobCommand::Balance { .. }
        | ClobCommand::UpdateBalance { .. }
        | ClobCommand::Notifications
        | ClobCommand::DeleteNotifications { .. }
        | ClobCommand::Rewards { .. }
        | ClobCommand::Earnings { .. }
        | ClobCommand::EarningsMarkets { .. }
        | ClobCommand::RewardPercentages
        | ClobCommand::CurrentRewards { .. }
        | ClobCommand::MarketReward { .. }
        | ClobCommand::OrderScoring { .. }
        | ClobCommand::OrdersScoring { .. }
        | ClobCommand::CreateApiKey => {
            unreachable!("execute() routes non-account commands to other handlers")
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_token_id_valid_numeric() {
        let id = parse_token_id("12345").unwrap();
        assert_eq!(id, U256::from(12345u64));
    }

    #[test]
    fn parse_token_id_large_number() {
        let id = parse_token_id(
            "48331043336612883890938759509493159234755048973583954730006854632066573",
        )
        .unwrap();
        assert!(id > U256::ZERO);
    }

    #[test]
    fn parse_token_id_zero() {
        let id = parse_token_id("0").unwrap();
        assert_eq!(id, U256::ZERO);
    }

    #[test]
    fn parse_token_id_invalid() {
        assert!(parse_token_id("abc").is_err());
        assert!(parse_token_id("12.34").is_err());
        assert!(parse_token_id("-1").is_err());
    }

    #[test]
    fn parse_token_ids_single() {
        let ids = parse_token_ids("100").unwrap();
        assert_eq!(ids, vec![U256::from(100u64)]);
    }

    #[test]
    fn parse_token_ids_multiple() {
        let ids = parse_token_ids("1,2,3").unwrap();
        assert_eq!(
            ids,
            vec![U256::from(1u64), U256::from(2u64), U256::from(3u64)]
        );
    }

    #[test]
    fn parse_token_ids_with_spaces() {
        let ids = parse_token_ids("1, 2, 3").unwrap();
        assert_eq!(
            ids,
            vec![U256::from(1u64), U256::from(2u64), U256::from(3u64)]
        );
    }

    #[test]
    fn parse_token_ids_invalid_entry() {
        assert!(parse_token_ids("1,abc,3").is_err());
    }

    #[test]
    fn parse_date_valid() {
        let d = parse_date("2024-06-15").unwrap();
        assert_eq!(d.to_string(), "2024-06-15");
    }

    #[test]
    fn parse_date_leap_day() {
        let d = parse_date("2024-02-29").unwrap();
        assert_eq!(d.to_string(), "2024-02-29");
    }

    #[test]
    fn parse_date_invalid_format() {
        assert!(parse_date("06/15/2024").is_err());
        assert!(parse_date("2024-13-01").is_err());
        assert!(parse_date("not-a-date").is_err());
        assert!(parse_date("").is_err());
    }
}
