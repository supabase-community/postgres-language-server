use assert_cmd::Command;
use insta::assert_snapshot;
use std::path::PathBuf;

const BIN: &str = "postgres-language-server";

fn run_check(args: &[&str]) -> String {
    let mut cmd = Command::cargo_bin(BIN).expect("binary not built");
    let mut full_args = vec!["check".to_string()];
    full_args.extend(args.iter().map(|s| s.to_string()));
    let test_sql = PathBuf::from("tests/fixtures/test.sql");
    full_args.push(test_sql.to_str().unwrap().to_string());

    let output = cmd.args(&full_args).output().expect("failed to run CLI");
    assert!(
        !output.status.success(),
        "command unexpectedly succeeded: {:?}",
        output.status
    );

    normalize_durations(&String::from_utf8_lossy(&output.stdout))
}

fn normalize_durations(input: &str) -> String {
    let mut content = input.to_owned();

    let mut search_start = 0;
    while let Some(relative) = content[search_start..].find(" in ") {
        let start = search_start + relative + 4;
        if let Some(end_rel) = content[start..].find('.') {
            let end = start + end_rel;
            if content[start..end].chars().any(|c| c.is_ascii_digit()) {
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
            if content[start..end].chars().any(|c| c.is_ascii_digit()) {
                content.replace_range(start..end, "<duration>");
            }
            time_search = end + 1;
        } else {
            break;
        }
    }

    content
}

#[test]
fn check_default_reporter_snapshot() {
    assert_snapshot!(run_check(&[]));
}

#[test]
fn check_github_reporter_snapshot() {
    assert_snapshot!(run_check(&["--reporter", "github"]));
}

#[test]
fn check_gitlab_reporter_snapshot() {
    assert_snapshot!(run_check(&["--reporter", "gitlab"]));
}

#[test]
fn check_junit_reporter_snapshot() {
    assert_snapshot!(run_check(&["--reporter", "junit"]));
}
