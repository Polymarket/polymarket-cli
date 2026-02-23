use super::{parse_address, parse_condition_id};
use crate::output::OutputFormat;
use crate::output::data::{
    print_activity, print_builder_leaderboard, print_builder_volume, print_closed_positions,
    print_holders, print_leaderboard, print_live_volume, print_open_interest, print_positions,
    print_traded, print_trades, print_value,
};
use anyhow::Result;
use clap::{Args, Subcommand};
use polymarket_client_sdk::data::{
    self,
    types::request::{
        ActivityRequest, BuilderLeaderboardRequest, BuilderVolumeRequest, ClosedPositionsRequest,
        HoldersRequest, LiveVolumeRequest, OpenInterestRequest, PositionsRequest, TradedRequest,
        TraderLeaderboardRequest, TradesRequest, ValueRequest,
    },
};

#[derive(Args)]
pub struct DataArgs {
    #[command(subcommand)]
    pub command: DataCommand,
}

#[derive(Subcommand)]
pub enum DataCommand {
    /// Get open positions for a wallet address
    Positions {
        /// Wallet address (0x...)
        address: String,

        /// Max results
        #[arg(long, default_value = "25")]
        limit: i32,

        /// Pagination offset
        #[arg(long)]
        offset: Option<i32>,
    },

    /// Get closed positions for a wallet address
    ClosedPositions {
        /// Wallet address (0x...)
        address: String,

        /// Max results
        #[arg(long, default_value = "25")]
        limit: i32,

        /// Pagination offset
        #[arg(long)]
        offset: Option<i32>,
    },

    /// Get total position value for a wallet address
    Value {
        /// Wallet address (0x...)
        address: String,
    },

    /// Get count of unique markets traded by a wallet
    Traded {
        /// Wallet address (0x...)
        address: String,
    },

    /// Get trade history
    Trades {
        /// Wallet address (0x...)
        address: String,

        /// Max results
        #[arg(long, default_value = "25")]
        limit: i32,

        /// Pagination offset
        #[arg(long)]
        offset: Option<i32>,
    },

    /// Get on-chain activity for a wallet address
    Activity {
        /// Wallet address (0x...)
        address: String,

        /// Max results
        #[arg(long, default_value = "25")]
        limit: i32,

        /// Pagination offset
        #[arg(long)]
        offset: Option<i32>,
    },

    /// Get top token holders for a market
    Holders {
        /// Market condition ID (0x...)
        market: String,

        /// Max results per token
        #[arg(long, default_value = "10")]
        limit: i32,
    },

    /// Get open interest for markets
    OpenInterest {
        /// Market condition ID (0x...)
        market: String,
    },

    /// Get live volume for an event
    Volume {
        /// Event ID
        id: u64,
    },

    /// Trader leaderboard
    Leaderboard {
        /// Time period: day, week, month, all
        #[arg(long)]
        period: Option<TimePeriod>,

        /// Order by: pnl or vol
        #[arg(long)]
        order_by: Option<OrderBy>,

        /// Max results
        #[arg(long, default_value = "25")]
        limit: i32,

        /// Pagination offset
        #[arg(long)]
        offset: Option<i32>,
    },

    /// Builder leaderboard
    BuilderLeaderboard {
        /// Time period: day, week, month, all
        #[arg(long)]
        period: Option<TimePeriod>,

        /// Max results
        #[arg(long, default_value = "25")]
        limit: i32,

        /// Pagination offset
        #[arg(long)]
        offset: Option<i32>,
    },

    /// Builder volume time-series
    BuilderVolume {
        /// Time period: day, week, month, all
        #[arg(long)]
        period: Option<TimePeriod>,
    },
}

#[derive(Clone, Debug, clap::ValueEnum)]
pub enum TimePeriod {
    Day,
    Week,
    Month,
    All,
}

impl From<TimePeriod> for polymarket_client_sdk::data::types::TimePeriod {
    fn from(t: TimePeriod) -> Self {
        match t {
            TimePeriod::Day => Self::Day,
            TimePeriod::Week => Self::Week,
            TimePeriod::Month => Self::Month,
            TimePeriod::All => Self::All,
        }
    }
}

#[derive(Clone, Debug, clap::ValueEnum)]
pub enum OrderBy {
    Pnl,
    Vol,
}

impl From<OrderBy> for polymarket_client_sdk::data::types::LeaderboardOrderBy {
    fn from(o: OrderBy) -> Self {
        match o {
            OrderBy::Pnl => Self::Pnl,
            OrderBy::Vol => Self::Vol,
        }
    }
}

#[allow(clippy::too_many_lines)]
pub async fn execute(client: &data::Client, args: DataArgs, output: OutputFormat) -> Result<()> {
    match args.command {
        DataCommand::Positions {
            address,
            limit,
            offset,
        } => {
            let request = PositionsRequest::builder()
                .user(parse_address(&address)?)
                .limit(limit)?
                .maybe_offset(offset)?
                .build();

            let positions = client.positions(&request).await?;
            print_positions(&positions, &output);
        }

        DataCommand::ClosedPositions {
            address,
            limit,
            offset,
        } => {
            let request = ClosedPositionsRequest::builder()
                .user(parse_address(&address)?)
                .limit(limit)?
                .maybe_offset(offset)?
                .build();

            let positions = client.closed_positions(&request).await?;
            print_closed_positions(&positions, &output);
        }

        DataCommand::Value { address } => {
            let request = ValueRequest::builder()
                .user(parse_address(&address)?)
                .build();

            let values = client.value(&request).await?;
            print_value(&values, &output);
        }

        DataCommand::Traded { address } => {
            let request = TradedRequest::builder()
                .user(parse_address(&address)?)
                .build();

            let traded = client.traded(&request).await?;
            print_traded(&traded, &output);
        }

        DataCommand::Trades {
            address,
            limit,
            offset,
        } => {
            let request = TradesRequest::builder()
                .user(parse_address(&address)?)
                .limit(limit)?
                .maybe_offset(offset)?
                .build();

            let trades = client.trades(&request).await?;
            print_trades(&trades, &output);
        }

        DataCommand::Activity {
            address,
            limit,
            offset,
        } => {
            let request = ActivityRequest::builder()
                .user(parse_address(&address)?)
                .limit(limit)?
                .maybe_offset(offset)?
                .build();

            let activity = client.activity(&request).await?;
            print_activity(&activity, &output);
        }

        DataCommand::Holders { market, limit } => {
            let cid = parse_condition_id(&market)?;
            let request = HoldersRequest::builder()
                .markets(vec![cid])
                .limit(limit)?
                .build();

            let holders = client.holders(&request).await?;
            print_holders(&holders, &output);
        }

        DataCommand::OpenInterest { market } => {
            let cid = parse_condition_id(&market)?;
            let request = OpenInterestRequest::builder().markets(vec![cid]).build();

            let oi = client.open_interest(&request).await?;
            print_open_interest(&oi, &output);
        }

        DataCommand::Volume { id } => {
            let request = LiveVolumeRequest::builder().id(id).build();
            let volume = client.live_volume(&request).await?;
            print_live_volume(&volume, &output);
        }

        DataCommand::Leaderboard {
            period,
            order_by,
            limit,
            offset,
        } => {
            let request = TraderLeaderboardRequest::builder()
                .maybe_time_period(period.map(Into::into))
                .maybe_order_by(order_by.map(Into::into))
                .limit(limit)?
                .maybe_offset(offset)?
                .build();

            let entries = client.leaderboard(&request).await?;
            print_leaderboard(&entries, &output);
        }

        DataCommand::BuilderLeaderboard {
            period,
            limit,
            offset,
        } => {
            let request = BuilderLeaderboardRequest::builder()
                .maybe_time_period(period.map(Into::into))
                .limit(limit)?
                .maybe_offset(offset)?
                .build();

            let entries = client.builder_leaderboard(&request).await?;
            print_builder_leaderboard(&entries, &output);
        }

        DataCommand::BuilderVolume { period } => {
            let request = BuilderVolumeRequest::builder()
                .maybe_time_period(period.map(Into::into))
                .build();

            let entries = client.builder_volume(&request).await?;
            print_builder_volume(&entries, &output);
        }
    }

    Ok(())
}
