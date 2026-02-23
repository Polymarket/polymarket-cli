use polymarket_client_sdk::gamma::types::response::Comment;
use tabled::settings::Style;
use tabled::{Table, Tabled};

use super::{detail_field, print_detail_table, truncate};

#[derive(Tabled)]
struct CommentRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Author")]
    author: String,
    #[tabled(rename = "Body")]
    body: String,
    #[tabled(rename = "Reactions")]
    reactions: String,
    #[tabled(rename = "Created")]
    created: String,
}

fn comment_author(c: &Comment) -> String {
    c.profile
        .as_ref()
        .and_then(|p| p.name.as_deref().or(p.pseudonym.as_deref()))
        .map(String::from)
        .or_else(|| c.user_address.map(|a| truncate(&format!("{a}"), 10)))
        .unwrap_or_else(|| "—".into())
}

fn comment_to_row(c: &Comment) -> CommentRow {
    CommentRow {
        id: truncate(&c.id, 12),
        author: comment_author(c),
        body: truncate(c.body.as_deref().unwrap_or("—"), 60),
        reactions: c
            .reaction_count
            .map_or_else(|| "—".into(), |n| n.to_string()),
        created: c
            .created_at
            .map_or_else(|| "—".into(), |d| d.format("%Y-%m-%d %H:%M").to_string()),
    }
}

pub fn print_comments_table(comments: &[Comment]) {
    if comments.is_empty() {
        println!("No comments found.");
        return;
    }
    let rows: Vec<CommentRow> = comments.iter().map(comment_to_row).collect();
    let table = Table::new(rows).with(Style::rounded()).to_string();
    println!("{table}");
}

pub fn print_comment_detail(c: &Comment) {
    let mut rows: Vec<[String; 2]> = Vec::new();

    detail_field!(rows, "ID", c.id.clone());
    detail_field!(rows, "Body", c.body.clone().unwrap_or_default());
    detail_field!(
        rows,
        "Entity Type",
        c.parent_entity_type.clone().unwrap_or_default()
    );
    detail_field!(
        rows,
        "Entity ID",
        c.parent_entity_id
            .map(|id| id.to_string())
            .unwrap_or_default()
    );
    detail_field!(
        rows,
        "Parent Comment",
        c.parent_comment_id.clone().unwrap_or_default()
    );
    detail_field!(
        rows,
        "User Address",
        c.user_address.map(|a| format!("{a}")).unwrap_or_default()
    );
    detail_field!(
        rows,
        "Author",
        c.profile
            .as_ref()
            .and_then(|p| p.name.as_deref().or(p.pseudonym.as_deref()))
            .unwrap_or_default()
            .into()
    );
    detail_field!(
        rows,
        "Reactions",
        c.reaction_count
            .map_or_else(|| "—".into(), |n| n.to_string())
    );
    detail_field!(
        rows,
        "Reports",
        c.report_count.map(|n| n.to_string()).unwrap_or_default()
    );
    detail_field!(
        rows,
        "Created At",
        c.created_at.map(|d| d.to_string()).unwrap_or_default()
    );
    detail_field!(
        rows,
        "Updated At",
        c.updated_at.map(|d| d.to_string()).unwrap_or_default()
    );

    print_detail_table(rows);
}
