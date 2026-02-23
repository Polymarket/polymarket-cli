use polymarket_client_sdk::gamma::types::response::{RelatedTag, Tag};
use tabled::settings::Style;
use tabled::{Table, Tabled};

use super::{detail_field, print_detail_table, truncate};

#[derive(Tabled)]
struct TagRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Label")]
    label: String,
    #[tabled(rename = "Slug")]
    slug: String,
    #[tabled(rename = "Carousel")]
    carousel: String,
}

fn tag_to_row(t: &Tag) -> TagRow {
    TagRow {
        id: truncate(&t.id, 20),
        label: t.label.as_deref().unwrap_or("—").into(),
        slug: t.slug.as_deref().unwrap_or("—").into(),
        carousel: t.is_carousel.map_or_else(|| "—".into(), |v| v.to_string()),
    }
}

pub fn print_tags_table(tags: &[Tag]) {
    if tags.is_empty() {
        println!("No tags found.");
        return;
    }
    let rows: Vec<TagRow> = tags.iter().map(tag_to_row).collect();
    let table = Table::new(rows).with(Style::rounded()).to_string();
    println!("{table}");
}

#[derive(Tabled)]
struct RelatedTagRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Tag ID")]
    tag_id: String,
    #[tabled(rename = "Related Tag ID")]
    related_tag_id: String,
    #[tabled(rename = "Rank")]
    rank: String,
}

fn related_tag_to_row(r: &RelatedTag) -> RelatedTagRow {
    RelatedTagRow {
        id: truncate(&r.id, 20),
        tag_id: r.tag_id.as_deref().unwrap_or("—").into(),
        related_tag_id: r.related_tag_id.as_deref().unwrap_or("—").into(),
        rank: r.rank.map_or_else(|| "—".into(), |v| v.to_string()),
    }
}

pub fn print_related_tags_table(tags: &[RelatedTag]) {
    if tags.is_empty() {
        println!("No related tags found.");
        return;
    }
    let rows: Vec<RelatedTagRow> = tags.iter().map(related_tag_to_row).collect();
    let table = Table::new(rows).with(Style::rounded()).to_string();
    println!("{table}");
}

#[allow(clippy::vec_init_then_push)]
pub fn print_tag_detail(t: &Tag) {
    let mut rows: Vec<[String; 2]> = Vec::new();

    detail_field!(rows, "ID", t.id.clone());
    detail_field!(rows, "Label", t.label.clone().unwrap_or_default());
    detail_field!(rows, "Slug", t.slug.clone().unwrap_or_default());
    detail_field!(
        rows,
        "Carousel",
        t.is_carousel.map(|v| v.to_string()).unwrap_or_default()
    );
    detail_field!(
        rows,
        "Force Show",
        t.force_show.map(|v| v.to_string()).unwrap_or_default()
    );
    detail_field!(
        rows,
        "Force Hide",
        t.force_hide.map(|v| v.to_string()).unwrap_or_default()
    );
    detail_field!(
        rows,
        "Created At",
        t.created_at.map(|d| d.to_string()).unwrap_or_default()
    );
    detail_field!(
        rows,
        "Updated At",
        t.updated_at.map(|d| d.to_string()).unwrap_or_default()
    );

    print_detail_table(rows);
}
