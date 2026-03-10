#![allow(deprecated)]

use assert_cmd::Command;
use predicates::prelude::*;

fn polymarket() -> Command {
    let mut cmd = Command::cargo_bin("polymarket").unwrap();
    cmd.env_remove("POLYMARKET_PRIVATE_KEY");
    cmd.env_remove("POLYMARKET_SIGNATURE_TYPE");
    cmd
}

#[test]
fn help_lists_all_top_level_commands() {
    polymarket().arg("--help").assert().success().stdout(
        predicate::str::contains("setup")
            .and(predicate::str::contains("shell"))
            .and(predicate::str::contains("markets"))
            .and(predicate::str::contains("events"))
            .and(predicate::str::contains("tags"))
            .and(predicate::str::contains("series"))
            .and(predicate::str::contains("comments"))
            .and(predicate::str::contains("profiles"))
            .and(predicate::str::contains("sports"))
            .and(predicate::str::contains("approve"))
            .and(predicate::str::contains("clob"))
            .and(predicate::str::contains("ctf"))
            .and(predicate::str::contains("data"))
            .and(predicate::str::contains("bridge"))
            .and(predicate::str::contains("wallet"))
            .and(predicate::str::contains("status")),
    );
}

#[test]
fn version_outputs_binary_name() {
    polymarket()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("polymarket"));
}

#[test]
fn markets_help_lists_subcommands() {
    polymarket()
        .args(["markets", "--help"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("list")
                .and(predicate::str::contains("get"))
                .and(predicate::str::contains("search"))
                .and(predicate::str::contains("tags")),
        );
}

#[test]
fn events_help_lists_subcommands() {
    polymarket()
        .args(["events", "--help"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("list")
                .and(predicate::str::contains("get"))
                .and(predicate::str::contains("tags")),
        );
}

#[test]
fn wallet_help_lists_subcommands() {
    polymarket()
        .args(["wallet", "--help"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("create")
                .and(predicate::str::contains("import"))
                .and(predicate::str::contains("address"))
                .and(predicate::str::contains("show"))
                .and(predicate::str::contains("reset")),
        );
}

#[test]
fn no_args_shows_usage() {
    polymarket()
        .assert()
        .failure()
        .stderr(predicate::str::contains("Usage"));
}

#[test]
fn unknown_command_fails() {
    polymarket().arg("nonexistent").assert().failure();
}

#[test]
fn invalid_output_format_rejected() {
    polymarket()
        .args(["--output", "xml", "status"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn markets_search_requires_query() {
    polymarket().args(["markets", "search"]).assert().failure();
}

#[test]
fn markets_get_requires_id() {
    polymarket().args(["markets", "get"]).assert().failure();
}

#[test]
fn comments_list_requires_entity_args() {
    polymarket().args(["comments", "list"]).assert().failure();
}

// Uses a guaranteed-to-fail command (nonexistent slug) to verify the error
// output contract: JSON mode → structured error on stdout, table mode → stderr.

#[test]
fn json_mode_error_is_valid_json_with_error_key() {
    let output = polymarket()
        .args([
            "--output",
            "json",
            "markets",
            "get",
            "nonexistent-slug-99999",
        ])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("stdout not valid JSON: {e}\nstdout: {stdout}"));
    assert!(
        parsed.get("error").is_some(),
        "missing 'error' key: {parsed}"
    );
}

#[test]
fn table_mode_error_goes_to_stderr() {
    polymarket()
        .args(["markets", "get", "nonexistent-slug-99999"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error:"));
}

#[test]
fn wallet_show_always_succeeds() {
    polymarket().args(["wallet", "show"]).assert().success();
}

#[test]
fn wallet_show_json_has_configured_field() {
    let output = polymarket()
        .args(["-o", "json", "wallet", "show"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("stdout not valid JSON: {e}\nstdout: {stdout}"));
    assert!(
        parsed.get("configured").is_some(),
        "missing 'configured' key: {parsed}"
    );
}

#[test]
fn tags_help_lists_subcommands() {
    polymarket()
        .args(["tags", "--help"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("list")
                .and(predicate::str::contains("get"))
                .and(predicate::str::contains("related")),
        );
}

#[test]
fn series_help_lists_subcommands() {
    polymarket()
        .args(["series", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list").and(predicate::str::contains("get")));
}

#[test]
fn comments_help_lists_subcommands() {
    polymarket()
        .args(["comments", "--help"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("list")
                .and(predicate::str::contains("get"))
                .and(predicate::str::contains("by-user")),
        );
}

#[test]
fn profiles_help_lists_subcommands() {
    polymarket()
        .args(["profiles", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("get"));
}

#[test]
fn sports_help_lists_subcommands() {
    polymarket()
        .args(["sports", "--help"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("list")
                .and(predicate::str::contains("market-types"))
                .and(predicate::str::contains("teams")),
        );
}

#[test]
fn clob_help_lists_subcommands() {
    polymarket()
        .args(["clob", "--help"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("book")
                .and(predicate::str::contains("price"))
                .and(predicate::str::contains("spread"))
                .and(predicate::str::contains("midpoint"))
                .and(predicate::str::contains("trades")),
        );
}

#[test]
fn data_help_lists_subcommands() {
    polymarket()
        .args(["data", "--help"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("positions")
                .and(predicate::str::contains("trades"))
                .and(predicate::str::contains("leaderboard")),
        );
}

#[test]
fn bridge_help_lists_subcommands() {
    polymarket()
        .args(["bridge", "--help"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("deposit")
                .and(predicate::str::contains("assets"))
                .and(predicate::str::contains("status")),
        );
}

#[test]
fn events_get_requires_id() {
    polymarket().args(["events", "get"]).assert().failure();
}

#[test]
fn tags_get_requires_id() {
    polymarket().args(["tags", "get"]).assert().failure();
}

#[test]
fn series_get_requires_id() {
    polymarket().args(["series", "get"]).assert().failure();
}

#[test]
fn comments_get_requires_id() {
    polymarket().args(["comments", "get"]).assert().failure();
}

#[test]
fn comments_by_user_requires_address() {
    polymarket()
        .args(["comments", "by-user"])
        .assert()
        .failure();
}

#[test]
fn profiles_get_requires_address() {
    polymarket().args(["profiles", "get"]).assert().failure();
}

#[test]
fn clob_book_requires_token() {
    polymarket().args(["clob", "book"]).assert().failure();
}

#[test]
fn clob_price_requires_token() {
    polymarket().args(["clob", "price"]).assert().failure();
}

#[test]
fn data_positions_requires_address() {
    polymarket().args(["data", "positions"]).assert().failure();
}

#[test]
fn approve_help_lists_subcommands() {
    polymarket()
        .args(["approve", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("check").and(predicate::str::contains("set")));
}

#[test]
fn ctf_help_lists_subcommands() {
    polymarket()
        .args(["ctf", "--help"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("split")
                .and(predicate::str::contains("merge"))
                .and(predicate::str::contains("redeem"))
                .and(predicate::str::contains("redeem-neg-risk"))
                .and(predicate::str::contains("condition-id"))
                .and(predicate::str::contains("collection-id"))
                .and(predicate::str::contains("position-id")),
        );
}

#[test]
fn ctf_collection_id_requires_condition_and_index_set() {
    polymarket()
        .args(["ctf", "collection-id"])
        .assert()
        .failure();
}

#[test]
fn ctf_collection_id_requires_index_set() {
    polymarket()
        .args([
            "ctf",
            "collection-id",
            "--condition",
            "0x0000000000000000000000000000000000000000000000000000000000000001",
        ])
        .assert()
        .failure();
}

#[test]
fn ctf_split_help_shows_all_flags() {
    polymarket()
        .args(["ctf", "split", "--help"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("--condition")
                .and(predicate::str::contains("--amount"))
                .and(predicate::str::contains("--collateral"))
                .and(predicate::str::contains("--partition"))
                .and(predicate::str::contains("--parent-collection")),
        );
}

#[test]
fn ctf_redeem_help_shows_index_sets_flag() {
    polymarket()
        .args(["ctf", "redeem", "--help"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("--index-sets")
                .and(predicate::str::contains("--collateral"))
                .and(predicate::str::contains("--parent-collection")),
        );
}

#[test]
fn ctf_split_requires_condition_and_amount() {
    polymarket().args(["ctf", "split"]).assert().failure();
}

#[test]
fn ctf_split_requires_amount() {
    polymarket()
        .args([
            "ctf",
            "split",
            "--condition",
            "0x0000000000000000000000000000000000000000000000000000000000000001",
        ])
        .assert()
        .failure();
}

#[test]
fn ctf_merge_requires_condition_and_amount() {
    polymarket().args(["ctf", "merge"]).assert().failure();
}

#[test]
fn ctf_redeem_requires_condition() {
    polymarket().args(["ctf", "redeem"]).assert().failure();
}

#[test]
fn ctf_redeem_neg_risk_requires_condition_and_amounts() {
    polymarket()
        .args(["ctf", "redeem-neg-risk"])
        .assert()
        .failure();
}

#[test]
fn ctf_condition_id_requires_all_args() {
    polymarket()
        .args(["ctf", "condition-id"])
        .assert()
        .failure();
}

#[test]
fn ctf_condition_id_requires_question() {
    polymarket()
        .args([
            "ctf",
            "condition-id",
            "--oracle",
            "0x0000000000000000000000000000000000000001",
            "--outcomes",
            "2",
        ])
        .assert()
        .failure();
}

#[test]
fn ctf_position_id_requires_collection() {
    polymarket().args(["ctf", "position-id"]).assert().failure();
}

#[test]
fn json_flag_short_form_works() {
    polymarket()
        .args(["-o", "json", "wallet", "show"])
        .assert()
        .success();
}

#[test]
fn table_output_is_default() {
    polymarket()
        .args(["wallet", "show"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Address").or(predicate::str::contains("configured")));
}

#[test]
fn wallet_address_succeeds_or_fails_gracefully() {
    // If no wallet configured, should fail with error; if configured, should succeed
    let output = polymarket().args(["wallet", "address"]).output().unwrap();
    // Either succeeds or fails with an error message — not a panic
    assert!(output.status.success() || !output.stderr.is_empty());
}

#[test]
fn watch_flag_appears_in_help() {
    polymarket()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--watch"));
}

#[test]
fn watch_with_json_output_rejected() {
    polymarket()
        .args(["--watch", "5", "-o", "json", "wallet", "show"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--watch is not supported with JSON"));
}

#[test]
fn watch_zero_interval_rejected() {
    polymarket()
        .args(["--watch", "0", "wallet", "show"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("at least 1 second"));
}
