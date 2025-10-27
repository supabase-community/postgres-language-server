use std::path::PathBuf;

use assert_cmd::Command;
use insta::assert_snapshot;
use std::process::ExitStatus;

const BIN: &str = "postgres-language-server";
const CONFIG_PATH: &str = "tests/fixtures/postgres-language-server.jsonc";

#[test]
#[cfg_attr(
    not(target_os = "linux"),
    ignore = "snapshot expectations only validated on Linux"
)]
fn test_cli_check_command() {
    let output = Command::cargo_bin(BIN)
        .unwrap()
        .args([
            "check",
            "--config-path",
            CONFIG_PATH,
            PathBuf::from("tests/fixtures/test.sql").to_str().unwrap(),
        ])
        .output()
        .unwrap();

    assert!(!output.status.success(), "command unexpectedly succeeded");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_snapshot!(normalize_output(output.status, &stdout, &stderr));
}

fn normalize_output(status: ExitStatus, stdout: &str, stderr: &str) -> String {
    let normalized_stdout = normalize_durations(stdout);
    let normalized_stderr = normalize_durations(stderr);
    let status_label = if status.success() {
        "success"
    } else {
        "failure"
    };
    format!(
        "status: {status_label}\nstdout:\n{}\nstderr:\n{}\n",
        normalized_stdout.trim_end(),
        normalized_stderr.trim_end()
    )
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
