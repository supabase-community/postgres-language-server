use assert_cmd::cargo_bin_cmd;
use insta::assert_snapshot;
use std::process::ExitStatus;

const CONFIG_PATH: &str = "tests/fixtures/postgres-language-server.jsonc";
const FORMATTED_PATH: &str = "tests/fixtures/format/formatted.sql";
const UNFORMATTED_PATH: &str = "tests/fixtures/format/unformatted.sql";
const BLANK_LINES_PATH: &str = "tests/fixtures/format/blank_lines.sql";

#[test]
#[cfg_attr(
    target_os = "windows",
    ignore = "snapshot expectations only validated on unix-like platforms"
)]
fn format_stdin_snapshot() {
    assert_snapshot!(run_format_stdin(
        &["--stdin-file-path", "virtual.sql"],
        "SELECT a,b FROM t;\n\n-- keep me\nselect 1\n"
    ));
}

#[test]
#[cfg_attr(
    target_os = "windows",
    ignore = "snapshot expectations only validated on unix-like platforms"
)]
fn format_stdin_with_syntax_error_snapshot() {
    assert_snapshot!(run_format_stdin(
        &["--stdin-file-path", "virtual.sql"],
        "SELECT a,b FROM t;\nselect from where;\n"
    ));
}

#[test]
#[cfg_attr(
    target_os = "windows",
    ignore = "snapshot expectations only validated on unix-like platforms"
)]
fn format_stdin_rejects_paths_snapshot() {
    assert_snapshot!(run_format_stdin(
        &["--stdin-file-path", "virtual.sql", UNFORMATTED_PATH],
        "select 1;\n"
    ));
}

#[test]
#[cfg_attr(
    target_os = "windows",
    ignore = "snapshot expectations only validated on unix-like platforms"
)]
fn format_stdin_rejects_write_snapshot() {
    assert_snapshot!(run_format_stdin(
        &["--stdin-file-path", "virtual.sql", "--write"],
        "select 1;\n"
    ));
}

#[test]
#[cfg_attr(
    target_os = "windows",
    ignore = "snapshot expectations only validated on unix-like platforms"
)]
fn format_without_write_reports_unformatted_files_snapshot() {
    assert_snapshot!(run_format(&[FORMATTED_PATH, UNFORMATTED_PATH]));
}

#[test]
#[cfg_attr(
    target_os = "windows",
    ignore = "snapshot expectations only validated on unix-like platforms"
)]
fn format_without_write_succeeds_on_formatted_files_snapshot() {
    assert_snapshot!(run_format(&[FORMATTED_PATH]));
}

#[test]
#[cfg_attr(
    target_os = "windows",
    ignore = "snapshot expectations only validated on unix-like platforms"
)]
fn format_without_write_reports_extra_blank_lines_snapshot() {
    assert_snapshot!(run_format(&[BLANK_LINES_PATH]));
}

fn run_format(args: &[&str]) -> String {
    run_format_with(args, None)
}

fn run_format_stdin(args: &[&str], stdin: &str) -> String {
    run_format_with(args, Some(stdin))
}

fn run_format_with(args: &[&str], stdin: Option<&str>) -> String {
    let mut cmd = cargo_bin_cmd!("postgres-language-server");
    if let Some(input) = stdin {
        cmd.write_stdin(input);
    }

    let mut full_args = vec![
        "format",
        "--config-path",
        CONFIG_PATH,
        "--log-level",
        "none",
    ];
    full_args.extend_from_slice(args);
    let output = cmd.args(full_args).output().expect("failed to run CLI");

    normalize_output(
        output.status,
        &String::from_utf8_lossy(&output.stdout),
        &String::from_utf8_lossy(&output.stderr),
    )
}

/// Replaces the duration in the summary line (e.g. `Processed 1 file in 3.2ms.`).
fn normalize_durations(input: &str) -> String {
    input
        .lines()
        .map(|line| {
            let Some(start) = line.find(" in ").filter(|_| line.starts_with("Processed ")) else {
                return line.to_string();
            };
            // The duration ends at the first period that is not a decimal point.
            let rest = &line[start + 4..];
            let end = rest
                .char_indices()
                .find(|&(i, c)| {
                    c == '.' && !rest[i + 1..].starts_with(|c: char| c.is_ascii_digit())
                })
                .map_or(rest.len(), |(i, _)| i);
            format!("{} in <duration>{}", &line[..start], &rest[end..])
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn normalize_output(status: ExitStatus, stdout: &str, stderr: &str) -> String {
    let status_label = if status.success() {
        "success"
    } else {
        "failure"
    };
    format!(
        "status: {status_label}\nstdout:\n{}\nstderr:\n{}\n",
        normalize_durations(stdout).trim_end(),
        stderr.trim_end()
    )
}
