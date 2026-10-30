use polymarket_client_sdk_v2::gamma::types::response::{
    SportsMarketTypesResponse, SportsMetadata, Team,
};
use tabled::settings::Style;
use tabled::{Table, Tabled};

use super::{DASH, OutputFormat, print_json, truncate};

#[derive(Tabled)]
struct SportRow {
    #[tabled(rename = "Sport")]
    sport: String,
    #[tabled(rename = "Resolution")]
    resolution: String,
    #[tabled(rename = "Series")]
    series: String,
    #[tabled(rename = "Tags")]
    tags: String,
}

fn sport_to_row(s: &SportsMetadata) -> SportRow {
    SportRow {
        sport: s.sport.clone(),
        resolution: truncate(&s.resolution, 40),
        series: s.series.clone(),
        tags: s.tags.join(", "),
    }
}

pub fn print_sports(sports: &[SportsMetadata], output: &OutputFormat) -> anyhow::Result<()> {
    match output {
        OutputFormat::Table => {
            if sports.is_empty() {
                println!("No sports found.");
                return Ok(());
            }
            let rows: Vec<SportRow> = sports.iter().map(sport_to_row).collect();
            let table = Table::new(rows).with(Style::rounded()).to_string();
            println!("{table}");
        }
        OutputFormat::Json => print_json(sports)?,
    }
    Ok(())
}

pub fn print_sport_types(
    types: &SportsMarketTypesResponse,
    output: &OutputFormat,
) -> anyhow::Result<()> {
    match output {
        OutputFormat::Table => {
            if types.market_types.is_empty() {
                println!("No market types found.");
                return Ok(());
            }
            let rows: Vec<[String; 1]> = types.market_types.iter().map(|t| [t.clone()]).collect();
            let table = Table::from_iter(rows).with(Style::rounded()).to_string();
            println!("{table}");
        }
        OutputFormat::Json => print_json(types)?,
    }
    Ok(())
}

#[derive(Tabled)]
struct TeamRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "League")]
    league: String,
    #[tabled(rename = "Record")]
    record: String,
    #[tabled(rename = "Abbreviation")]
    abbreviation: String,
}

fn team_to_row(t: &Team) -> TeamRow {
    TeamRow {
        id: t.id.to_string(),
        name: t.name.as_deref().unwrap_or(DASH).into(),
        league: t.league.as_deref().unwrap_or(DASH).into(),
        record: t.record.as_deref().unwrap_or(DASH).into(),
        abbreviation: t.abbreviation.as_deref().unwrap_or(DASH).into(),
    }
}

pub fn print_teams(teams: &[Team], output: &OutputFormat) -> anyhow::Result<()> {
    match output {
        OutputFormat::Table => {
            if teams.is_empty() {
                println!("No teams found.");
                return Ok(());
            }
            let rows: Vec<TeamRow> = teams.iter().map(team_to_row).collect();
            let table = Table::new(rows).with(Style::rounded()).to_string();
            println!("{table}");
        }
        OutputFormat::Json => print_json(teams)?,
    }
    Ok(())
}
