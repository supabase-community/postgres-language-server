use assert_cmd::Command;
use std::process::ExitStatus;

const BIN: &str = "postgres-language-server";

/// Get database URL from environment or use default docker-compose URL
fn get_database_url() -> Option<String> {
    std::env::var("DATABASE_URL")
        .ok()
        .or_else(|| Some("postgres://postgres:postgres@127.0.0.1:5432/postgres".to_string()))
}

/// Execute SQL against the database
fn execute_sql(sql: &str) -> bool {
    let Some(url) = get_database_url() else {
        return false;
    };

    std::process::Command::new("psql")
        .arg(&url)
        .arg("-c")
        .arg(sql)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Setup test schema with known issues for splinter to detect
fn setup_test_schema() {
    // Create a table without a primary key (triggers no_primary_key rule)
    execute_sql("DROP TABLE IF EXISTS dblint_test_no_pk CASCADE");
    execute_sql("CREATE TABLE dblint_test_no_pk (id int, name text)");
}

/// Cleanup test schema
fn cleanup_test_schema() {
    execute_sql("DROP TABLE IF EXISTS dblint_test_no_pk CASCADE");
}

#[test]
#[cfg_attr(
    target_os = "windows",
    ignore = "snapshot expectations only validated on unix-like platforms"
)]
fn dblint_runs_without_errors() {
    let output = run_dblint(&[]);
    assert!(
        output.contains("Command completed"),
        "Expected successful completion, got: {output}",
    );
}

#[test]
#[cfg_attr(
    target_os = "windows",
    ignore = "snapshot expectations only validated on unix-like platforms"
)]
fn dblint_detects_no_primary_key() {
    // Setup: create table without primary key
    setup_test_schema();

    // Run dblint
    let output = run_dblint(&[]);

    // Cleanup
    cleanup_test_schema();

    // Should detect the no_primary_key issue
    assert!(
        output.contains("noPrimaryKey") || output.contains("primary key"),
        "Expected to detect missing primary key issue, got: {output}",
    );
}

#[test]
#[cfg_attr(
    target_os = "windows",
    ignore = "snapshot expectations only validated on unix-like platforms"
)]
fn dblint_fails_without_database() {
    // Test that dblint fails gracefully when no database is configured
    let mut cmd = Command::cargo_bin(BIN).expect("binary not built");
    let output = cmd
        .args(["dblint", "--disable-db", "--log-level", "none"])
        .output()
        .expect("failed to run CLI");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should complete (possibly with warning about no database)
    assert!(
        output.status.success()
            || stderr.contains("database")
            || stdout.contains("Command completed"),
        "Expected graceful handling without database, got stdout: {stdout}, stderr: {stderr}",
    );
}

fn run_dblint(args: &[&str]) -> String {
    let url = get_database_url().expect("database URL required");

    let mut cmd = Command::cargo_bin(BIN).expect("binary not built");
    let mut full_args = vec!["dblint", "--connection-string", &url, "--log-level", "none"];
    full_args.extend_from_slice(args);

    let output = cmd.args(full_args).output().expect("failed to run CLI");

    normalize_output(
        output.status,
        &String::from_utf8_lossy(&output.stdout),
        &String::from_utf8_lossy(&output.stderr),
    )
}

fn normalize_output(status: ExitStatus, stdout: &str, stderr: &str) -> String {
    let normalized_stdout = normalize_durations(stdout);
    let status_label = if status.success() {
        "success"
    } else {
        "failure"
    };
    format!(
        "status: {status_label}\nstdout:\n{}\nstderr:\n{}\n",
        normalized_stdout.trim_end(),
        stderr.trim_end()
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

    content
}
