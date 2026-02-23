use super::parse_address;
use crate::output::profiles::print_profile_detail;
use crate::output::{OutputFormat, print_json};
use anyhow::Result;
use clap::{Args, Subcommand};
use polymarket_client_sdk::gamma::{self, types::request::PublicProfileRequest};

#[derive(Args)]
pub struct ProfilesArgs {
    #[command(subcommand)]
    pub command: ProfilesCommand,
}

#[derive(Subcommand)]
pub enum ProfilesCommand {
    /// Get a public profile by wallet address
    Get {
        /// Wallet address (0x...)
        address: String,
    },
}

pub async fn execute(
    client: &gamma::Client,
    args: ProfilesArgs,
    output: OutputFormat,
) -> Result<()> {
    match args.command {
        ProfilesCommand::Get { address } => {
            let addr = parse_address(&address)?;
            let req = PublicProfileRequest::builder().address(addr).build();
            let profile = client.public_profile(&req).await?;

            match output {
                OutputFormat::Table => print_profile_detail(&profile),
                OutputFormat::Json => print_json(&profile)?,
            }
        }
    }

    Ok(())
}
