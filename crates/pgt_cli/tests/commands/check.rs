use assert_cmd::Command;
use insta::assert_snapshot;
use std::path::Path;
use std::process::ExitStatus;

const BIN: &str = "postgres-language-server";
const CONFIG_PATH: &str = "tests/fixtures/postgres-language-server.jsonc";

fn run_check(args: &[&str]) -> String {
    let mut full_args = vec!["check", "--config-path", CONFIG_PATH];
    full_args.extend_from_slice(args);
    run_check_with(&full_args, None, None)
}

fn run_check_with(args: &[&str], stdin: Option<&str>, cwd: Option<&Path>) -> String {
    let mut cmd = Command::cargo_bin(BIN).expect("binary not built");
    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }
    if let Some(input) = stdin {
        cmd.write_stdin(input);
    }

    let output = cmd.args(args).output().expect("failed to run CLI");

    normalize_output(
        output.status,
        &String::from_utf8_lossy(&output.stdout),
        &String::from_utf8_lossy(&output.stderr),
    )
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

#[test]
#[cfg_attr(
    not(target_os = "linux"),
    ignore = "snapshot expectations only validated on Linux"
)]
fn check_default_reporter_snapshot() {
    assert_snapshot!(run_check(&["tests/fixtures/test.sql"]));
}

#[test]
#[cfg_attr(
    not(target_os = "linux"),
    ignore = "snapshot expectations only validated on Linux"
)]
fn check_github_reporter_snapshot() {
    assert_snapshot!(run_check(&[
        "--reporter",
        "github",
        "tests/fixtures/test.sql"
    ]));
}

#[test]
#[cfg_attr(
    not(target_os = "linux"),
    ignore = "snapshot expectations only validated on Linux"
)]
fn check_gitlab_reporter_snapshot() {
    assert_snapshot!(run_check(&[
        "--reporter",
        "gitlab",
        "tests/fixtures/test.sql"
    ]));
}

#[test]
#[cfg_attr(
    not(target_os = "linux"),
    ignore = "snapshot expectations only validated on Linux"
)]
fn check_junit_reporter_snapshot() {
    assert_snapshot!(run_check(&[
        "--reporter",
        "junit",
        "tests/fixtures/test.sql"
    ]));
}

#[test]
#[cfg_attr(
    not(target_os = "linux"),
    ignore = "snapshot expectations only validated on Linux"
)]
fn check_stdin_snapshot() {
    assert_snapshot!(run_check_with(
        &[
            "check",
            "--config-path",
            CONFIG_PATH,
            "--stdin-file-path",
            "virtual.sql"
        ],
        Some("alter tqjable stdin drop column id;\n"),
        None
    ));
}

#[test]
#[cfg_attr(
    not(target_os = "linux"),
    ignore = "snapshot expectations only validated on Linux"
)]
fn check_directory_traversal_snapshot() {
    let project_dir = Path::new("tests/fixtures/traversal");
    assert_snapshot!(run_check_with(
        &["check", "--diagnostic-level", "info", "."],
        None,
        Some(project_dir)
    ));
}
