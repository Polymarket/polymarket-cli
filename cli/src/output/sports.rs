use polymarket_client_sdk::gamma::types::response::{
    SportsMarketTypesResponse, SportsMetadata, Team,
};
use tabled::settings::Style;
use tabled::{Table, Tabled};

use super::truncate;

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

pub fn print_sports_table(sports: &[SportsMetadata]) {
    if sports.is_empty() {
        println!("No sports found.");
        return;
    }
    let rows: Vec<SportRow> = sports.iter().map(sport_to_row).collect();
    let table = Table::new(rows).with(Style::rounded()).to_string();
    println!("{table}");
}

pub fn print_sport_types(types: &SportsMarketTypesResponse) {
    if types.market_types.is_empty() {
        println!("No market types found.");
        return;
    }
    let rows: Vec<[String; 1]> = types.market_types.iter().map(|t| [t.clone()]).collect();
    let table = Table::from_iter(rows).with(Style::rounded()).to_string();
    println!("{table}");
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
        name: t.name.as_deref().unwrap_or("—").into(),
        league: t.league.as_deref().unwrap_or("—").into(),
        record: t.record.as_deref().unwrap_or("—").into(),
        abbreviation: t.abbreviation.as_deref().unwrap_or("—").into(),
    }
}

pub fn print_teams_table(teams: &[Team]) {
    if teams.is_empty() {
        println!("No teams found.");
        return;
    }
    let rows: Vec<TeamRow> = teams.iter().map(team_to_row).collect();
    let table = Table::new(rows).with(Style::rounded()).to_string();
    println!("{table}");
}
