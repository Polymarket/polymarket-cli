use anyhow::Result;
use clap::{Args, Subcommand};
use polymarket_client_sdk::gamma::{
    self,
    types::request::{SeriesByIdRequest, SeriesListRequest},
};

use crate::output::series::{print_series_detail, print_series_table};
use crate::output::{OutputFormat, print_json};

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
                .maybe_ascending(if ascending { Some(true) } else { None })
                .maybe_closed(closed)
                .build();

            let series = client.series(&request).await?;

            match output {
                OutputFormat::Table => print_series_table(&series),
                OutputFormat::Json => print_json(&series)?,
            }
        }

        SeriesCommand::Get { id } => {
            let req = SeriesByIdRequest::builder().id(id).build();
            let series = client.series_by_id(&req).await?;

            match output {
                OutputFormat::Table => print_series_detail(&series),
                OutputFormat::Json => print_json(&series)?,
            }
        }
    }

    Ok(())
}
