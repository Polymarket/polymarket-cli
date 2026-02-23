use anyhow::Result;
use clap::{Args, Subcommand};
use polymarket_client_sdk::gamma::{
    self,
    types::request::{
        RelatedTagsByIdRequest, RelatedTagsBySlugRequest, TagByIdRequest, TagBySlugRequest,
        TagsRequest,
    },
};

use super::is_numeric_id;
use crate::output::tags::{print_related_tags_table, print_tag_detail, print_tags_table};
use crate::output::{OutputFormat, print_json};

#[derive(Args)]
pub struct TagsArgs {
    #[command(subcommand)]
    pub command: TagsCommand,
}

#[derive(Subcommand)]
pub enum TagsCommand {
    /// List tags
    List {
        /// Max results
        #[arg(long, default_value = "25")]
        limit: i32,

        /// Pagination offset
        #[arg(long)]
        offset: Option<i32>,

        /// Sort ascending instead of descending
        #[arg(long)]
        ascending: bool,
    },

    /// Get a single tag by ID or slug
    Get {
        /// Tag ID or slug
        id: String,
    },

    /// Get related tag relationships for a tag
    Related {
        /// Tag ID or slug
        id: String,

        /// Omit empty related tags
        #[arg(long)]
        omit_empty: Option<bool>,
    },

    /// Get actual tag objects related to a tag
    RelatedTags {
        /// Tag ID or slug
        id: String,

        /// Omit empty related tags
        #[arg(long)]
        omit_empty: Option<bool>,
    },
}

pub async fn execute(client: &gamma::Client, args: TagsArgs, output: OutputFormat) -> Result<()> {
    match args.command {
        TagsCommand::List {
            limit,
            offset,
            ascending,
        } => {
            let request = TagsRequest::builder()
                .limit(limit)
                .maybe_offset(offset)
                .maybe_ascending(if ascending { Some(true) } else { None })
                .build();

            let tags = client.tags(&request).await?;

            match output {
                OutputFormat::Table => print_tags_table(&tags),
                OutputFormat::Json => print_json(&tags)?,
            }
        }

        TagsCommand::Get { id } => {
            let is_numeric = is_numeric_id(&id);
            let tag = if is_numeric {
                let req = TagByIdRequest::builder().id(id).build();
                client.tag_by_id(&req).await?
            } else {
                let req = TagBySlugRequest::builder().slug(id).build();
                client.tag_by_slug(&req).await?
            };

            match output {
                OutputFormat::Table => print_tag_detail(&tag),
                OutputFormat::Json => print_json(&tag)?,
            }
        }

        TagsCommand::Related { id, omit_empty } => {
            let is_numeric = is_numeric_id(&id);
            let related = if is_numeric {
                let req = RelatedTagsByIdRequest::builder()
                    .id(id)
                    .maybe_omit_empty(omit_empty)
                    .build();
                client.related_tags_by_id(&req).await?
            } else {
                let req = RelatedTagsBySlugRequest::builder()
                    .slug(id)
                    .maybe_omit_empty(omit_empty)
                    .build();
                client.related_tags_by_slug(&req).await?
            };

            match output {
                OutputFormat::Table => print_related_tags_table(&related),
                OutputFormat::Json => print_json(&related)?,
            }
        }

        TagsCommand::RelatedTags { id, omit_empty } => {
            let is_numeric = is_numeric_id(&id);
            let tags = if is_numeric {
                let req = RelatedTagsByIdRequest::builder()
                    .id(id)
                    .maybe_omit_empty(omit_empty)
                    .build();
                client.tags_related_to_tag_by_id(&req).await?
            } else {
                let req = RelatedTagsBySlugRequest::builder()
                    .slug(id)
                    .maybe_omit_empty(omit_empty)
                    .build();
                client.tags_related_to_tag_by_slug(&req).await?
            };

            match output {
                OutputFormat::Table => print_tags_table(&tags),
                OutputFormat::Json => print_json(&tags)?,
            }
        }
    }

    Ok(())
}
