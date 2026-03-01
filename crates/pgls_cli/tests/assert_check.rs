use assert_cmd::Command;
use insta::assert_snapshot;
use std::path::Path;
use std::process::ExitStatus;

const BIN: &str = "postgres-language-server";
const CONFIG_PATH: &str = "tests/fixtures/postgres-language-server.jsonc";

#[test]
#[cfg_attr(
    target_os = "windows",
    ignore = "snapshot expectations only validated on unix-like platforms"
)]
fn check_default_reporter_snapshot() {
    assert_snapshot!(run_check(&["tests/fixtures/test.sql"]));
}

#[test]
#[cfg_attr(
    target_os = "windows",
    ignore = "snapshot expectations only validated on unix-like platforms"
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
    target_os = "windows",
    ignore = "snapshot expectations only validated on unix-like platforms"
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
    target_os = "windows",
    ignore = "snapshot expectations only validated on unix-like platforms"
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
    target_os = "windows",
    ignore = "snapshot expectations only validated on unix-like platforms"
)]
fn check_json_reporter_snapshot() {
    assert_snapshot!(run_check(&[
        "--reporter",
        "json",
        "tests/fixtures/test.sql"
    ]));
}

#[test]
#[cfg_attr(
    target_os = "windows",
    ignore = "snapshot expectations only validated on unix-like platforms"
)]
fn check_json_pretty_reporter_snapshot() {
    assert_snapshot!(run_check(&[
        "--reporter",
        "json-pretty",
        "tests/fixtures/test.sql"
    ]));
}

#[test]
#[cfg_attr(
    target_os = "windows",
    ignore = "snapshot expectations only validated on unix-like platforms"
)]
fn check_summary_reporter_snapshot() {
    assert_snapshot!(run_check(&[
        "--reporter",
        "summary",
        "tests/fixtures/test.sql"
    ]));
}

#[test]
#[cfg_attr(
    target_os = "windows",
    ignore = "snapshot expectations only validated on unix-like platforms"
)]
fn check_stdin_snapshot() {
    assert_snapshot!(run_check_with(
        &[
            "--config-path",
            CONFIG_PATH,
            "--stdin-file-path",
            "virtual.sql",
            "--log-level",
            "none"
        ],
        Some("alter tqjable stdin drop column id;\n"),
        None
    ));
}

#[test]
#[cfg_attr(
    target_os = "windows",
    ignore = "snapshot expectations only validated on unix-like platforms"
)]
fn check_directory_traversal_snapshot() {
    let project_dir = Path::new("tests/fixtures/traversal");
    assert_snapshot!(run_check_with(
        &["--diagnostic-level", "info", "."],
        None,
        Some(project_dir)
    ));
}

fn run_check(args: &[&str]) -> String {
    let mut full_args = vec!["--config-path", CONFIG_PATH, "--log-level", "none"];
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

    let mut full_args = vec!["check"];
    full_args.extend_from_slice(args);
    let output = cmd.args(full_args).output().expect("failed to run CLI");

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
        // Find end of sentence period: scan for '.' that is followed by
        // a non-digit (or EOL), skipping decimal points inside durations.
        let rest = &content[start..];
        let mut dot_search = 0;
        let mut found_end = None;
        while let Some(dot_rel) = rest[dot_search..].find('.') {
            let dot_pos = dot_search + dot_rel;
            let after_dot = dot_pos + 1;
            if after_dot >= rest.len() || !rest.as_bytes()[after_dot].is_ascii_digit() {
                found_end = Some(start + dot_pos);
                break;
            }
            dot_search = after_dot;
        }
        if let Some(end) = found_end {
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

    // Normalize JSON "duration":12345 and "duration": 12345 (numeric nanos)
    for json_dur_pat in &["\"duration\":", "\"duration\": "] {
        let mut json_search = 0;
        while let Some(relative) = content[json_search..].find(json_dur_pat) {
            let start = json_search + relative + json_dur_pat.len();
            let rest = &content[start..];
            // Skip if it's a string value (already handled or not numeric)
            if rest.starts_with('"') {
                json_search = start + 1;
                continue;
            }
            let num_end = rest
                .find(|c: char| !c.is_ascii_digit())
                .unwrap_or(rest.len());
            if num_end > 0 {
                content.replace_range(start..start + num_end, "0");
                json_search = start + 1;
            } else {
                json_search = start + 1;
            }
        }
    }

    // Normalize Rust Debug durations (e.g. "4.877792ms", "1.23s", "123µs", "123ns")
    // used by summary reporter: "file(s) in 4.877ms." / "Completed in 4.877ms."
    for prefix in &["file(s) in ", "Completed in "] {
        let mut search = 0;
        while let Some(relative) = content[search..].find(prefix) {
            let start = search + relative + prefix.len();
            // Find the end of the duration: digits, '.', and time unit suffix
            let rest = &content[start..];
            let dur_end = rest
                .find(|c: char| !c.is_ascii_digit() && c != '.' && c != 'µ')
                .unwrap_or(rest.len());
            // Include trailing unit letters (m, s, n, etc.)
            let after_digits = &rest[dur_end..];
            let unit_end = after_digits
                .find(|c: char| !c.is_ascii_lowercase())
                .unwrap_or(after_digits.len());
            let total_end = start + dur_end + unit_end;
            if total_end > start {
                content.replace_range(start..total_end, "<duration>");
                search = start + "<duration>".len();
            } else {
                search = start + 1;
            }
        }
    }

    content
}

fn normalize_output(status: ExitStatus, stdout: &str, stderr: &str) -> String {
    let normalized_stdout = normalize_durations(stdout);
    let normalized_stderr = normalize_diagnostics(stderr);
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

fn normalize_diagnostics(input: &str) -> String {
    let normalized = normalize_durations(input);
    let mut lines = normalized.lines().peekable();
    let mut diagnostic_sections: Vec<String> = Vec::new();
    let mut other_lines: Vec<String> = Vec::new();

    while let Some(line) = lines.next() {
        if is_path_line(line) {
            let mut block = String::from(line);
            while let Some(&next) = lines.peek() {
                if is_path_line(next) || next.starts_with("check ") {
                    break;
                }
                block.push('\n');
                block.push_str(next);
                lines.next();
            }
            diagnostic_sections.push(trim_trailing_newlines(block));
        } else {
            other_lines.push(line.to_string());
        }
    }

    diagnostic_sections.sort();

    let mut parts = Vec::new();
    if !diagnostic_sections.is_empty() {
        parts.push(diagnostic_sections.join("\n\n"));
    }

    let rest = trim_trailing_newlines(other_lines.join("\n"));
    if rest.trim().is_empty() {
        parts.join("\n\n")
    } else if parts.is_empty() {
        rest
    } else {
        parts.push(rest);
        parts.join("\n\n")
    }
}

fn is_path_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    (trimmed.starts_with("./") || trimmed.starts_with("tests/"))
        && trimmed.contains(':')
        && trimmed.contains(" syntax")
}

fn trim_trailing_newlines(mut value: String) -> String {
    while value.ends_with('\n') {
        value.pop();
    }
    value
}
