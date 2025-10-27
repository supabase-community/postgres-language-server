use std::path::PathBuf;

use assert_cmd::Command;
use insta::assert_snapshot;

const BIN: &str = "postgres-language-server";

#[test]
fn test_cli_check_command() {
    let output = Command::cargo_bin(BIN)
        .unwrap()
        .args([
            "check",
            PathBuf::from("tests/fixtures/test.sql").to_str().unwrap(),
        ])
        .output()
        .unwrap();

    assert!(!output.status.success(), "command unexpectedly succeeded");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    assert_snapshot!(normalize_durations(&stdout));
}

fn normalize_durations(input: &str) -> String {
    let mut content = input.to_owned();
    let mut search_start = 0;
    while let Some(relative) = content[search_start..].find(" in ") {
        let start = search_start + relative + 4;
        if let Some(end_rel) = content[start..].find('.') {
            let end = start + end_rel;
            let slice = &content[start..end];
            if slice.chars().any(|c| c.is_ascii_digit()) {
                content.replace_range(start..end, "<duration>");
                search_start = start + "<duration>".len() + 1;
                continue;
            }
            search_start = end + 1;
        } else {
            break;
        }
    }
    let mut time_search = 0;
    while let Some(relative) = content[time_search..].find("time=\"") {
        let start = time_search + relative + 6;
        if let Some(end_rel) = content[start..].find('"') {
            let end = start + end_rel;
            let slice = &content[start..end];
            if slice.chars().any(|c| c.is_ascii_digit()) {
                content.replace_range(start..end, "<duration>");
            }
            time_search = end + 1;
        } else {
            break;
        }
    }
    content
}
