use anyhow::Result;
use clap::{Args, Subcommand};
use polymarket_client_sdk::gamma::{
    self,
    types::request::{SeriesByIdRequest, SeriesListRequest},
};

use crate::output::OutputFormat;
use crate::output::series::{print_series, print_series_item};

#[derive(Args)]
pub struct SeriesArgs {
    #[command(subcommand)]
    pub command: SeriesCommand,
}

#[derive(Subcommand)]
pub enum SeriesCommand {
    /// List series
    List {
        /// Max results
        #[arg(long, default_value = "25")]
        limit: i32,

        /// Pagination offset
        #[arg(long)]
        offset: Option<i32>,

        /// Sort field (e.g. volume, liquidity)
        #[arg(long)]
        order: Option<String>,

        /// Sort ascending instead of descending
        #[arg(long)]
        ascending: bool,

        /// Filter by closed status
        #[arg(long)]
        closed: Option<bool>,
    },

    /// Get a single series by ID
    Get {
        /// Series ID
        id: String,
    },
}

pub async fn execute(client: &gamma::Client, args: SeriesArgs, output: OutputFormat) -> Result<()> {
    match args.command {
        SeriesCommand::List {
            limit,
            offset,
            order,
            ascending,
            closed,
        } => {
            let request = SeriesListRequest::builder()
                .limit(limit)
                .maybe_offset(offset)
                .maybe_order(order)
                .ascending(ascending)
                .maybe_closed(closed)
                .build();

            let series = client.series(&request).await?;
            print_series(&series, &output)?;
        }

        SeriesCommand::Get { id } => {
            let req = SeriesByIdRequest::builder().id(id).build();
            let series = client.series_by_id(&req).await?;

            print_series_item(&series, &output)?;
        }
    }

    Ok(())
}
