use super::parse_address;
use crate::output::comments::{print_comment_detail, print_comments_table};
use crate::output::{OutputFormat, print_json};
use anyhow::Result;
use clap::{Args, Subcommand};
use polymarket_client_sdk::gamma::{
    self,
    types::{
        ParentEntityType,
        request::{CommentsByIdRequest, CommentsByUserAddressRequest, CommentsRequest},
    },
};

#[derive(Args)]
pub struct CommentsArgs {
    #[command(subcommand)]
    pub command: CommentsCommand,
}

#[derive(Subcommand)]
pub enum CommentsCommand {
    /// List comments on an event, market, or series
    List {
        /// Parent entity type: event, market, or series
        #[arg(long)]
        entity_type: EntityType,

        /// Parent entity ID
        #[arg(long)]
        entity_id: String,

        /// Max results
        #[arg(long, default_value = "25")]
        limit: i32,

        /// Pagination offset
        #[arg(long)]
        offset: Option<i32>,

        /// Sort field
        #[arg(long)]
        order: Option<String>,

        /// Sort ascending instead of descending
        #[arg(long)]
        ascending: bool,
    },

    /// Get a comment by ID
    Get {
        /// Comment ID
        id: String,
    },

    /// List comments by a user's wallet address
    ByUser {
        /// Wallet address (0x...)
        address: String,

        /// Max results
        #[arg(long, default_value = "25")]
        limit: i32,

        /// Pagination offset
        #[arg(long)]
        offset: Option<i32>,

        /// Sort field
        #[arg(long)]
        order: Option<String>,

        /// Sort ascending instead of descending
        #[arg(long)]
        ascending: bool,
    },
}

#[derive(Clone, Debug, clap::ValueEnum)]
pub enum EntityType {
    Event,
    Market,
    Series,
}

impl From<EntityType> for ParentEntityType {
    fn from(e: EntityType) -> Self {
        match e {
            EntityType::Event => ParentEntityType::Event,
            EntityType::Market => ParentEntityType::Market,
            EntityType::Series => ParentEntityType::Series,
        }
    }
}

pub async fn execute(
    client: &gamma::Client,
    args: CommentsArgs,
    output: OutputFormat,
) -> Result<()> {
    match args.command {
        CommentsCommand::List {
            entity_type,
            entity_id,
            limit,
            offset,
            order,
            ascending,
        } => {
            let request = CommentsRequest::builder()
                .parent_entity_type(ParentEntityType::from(entity_type))
                .parent_entity_id(entity_id)
                .limit(limit)
                .maybe_offset(offset)
                .maybe_order(order)
                .maybe_ascending(if ascending { Some(true) } else { None })
                .build();

            let comments = client.comments(&request).await?;

            match output {
                OutputFormat::Table => print_comments_table(&comments),
                OutputFormat::Json => print_json(&comments)?,
            }
        }

        CommentsCommand::Get { id } => {
            let req = CommentsByIdRequest::builder().id(id).build();
            let comments = client.comments_by_id(&req).await?;

            let Some(comment) = comments.first() else {
                anyhow::bail!("Comment not found");
            };

            match output {
                OutputFormat::Table => print_comment_detail(comment),
                OutputFormat::Json => print_json(&comment)?,
            }
        }

        CommentsCommand::ByUser {
            address,
            limit,
            offset,
            order,
            ascending,
        } => {
            let addr = parse_address(&address)?;
            let request = CommentsByUserAddressRequest::builder()
                .user_address(addr)
                .limit(limit)
                .maybe_offset(offset)
                .maybe_order(order)
                .maybe_ascending(if ascending { Some(true) } else { None })
                .build();

            let comments = client.comments_by_user_address(&request).await?;

            match output {
                OutputFormat::Table => print_comments_table(&comments),
                OutputFormat::Json => print_json(&comments)?,
            }
        }
    }

    Ok(())
}
