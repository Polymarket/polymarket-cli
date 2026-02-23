use polymarket_client_sdk::gamma::types::response::PublicProfile;

use super::{detail_field, print_detail_table};

pub fn print_profile_detail(p: &PublicProfile) {
    let mut rows: Vec<[String; 2]> = Vec::new();

    detail_field!(rows, "Name", p.name.clone().unwrap_or_default());
    detail_field!(rows, "Pseudonym", p.pseudonym.clone().unwrap_or_default());
    detail_field!(rows, "Bio", p.bio.clone().unwrap_or_default());
    detail_field!(
        rows,
        "Proxy Wallet",
        p.proxy_wallet.map(|a| format!("{a}")).unwrap_or_default()
    );
    detail_field!(
        rows,
        "Profile Image",
        p.profile_image.clone().unwrap_or_default()
    );
    detail_field!(rows, "X Username", p.x_username.clone().unwrap_or_default());
    detail_field!(
        rows,
        "Verified",
        p.verified_badge.map(|v| v.to_string()).unwrap_or_default()
    );
    detail_field!(
        rows,
        "Public Username",
        p.display_username_public
            .map(|v| v.to_string())
            .unwrap_or_default()
    );
    detail_field!(
        rows,
        "Created At",
        p.created_at.map(|d| d.to_string()).unwrap_or_default()
    );

    print_detail_table(rows);
}
