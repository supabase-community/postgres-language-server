use assert_cmd::Command;
use insta::assert_snapshot;
use sqlx::PgPool;
use std::process::ExitStatus;

const BIN: &str = "postgres-language-server";

/// Get database URL from the pool's connect options
/// Uses the known docker-compose credentials (postgres:postgres)
fn get_database_url(pool: &PgPool) -> String {
    let opts = pool.connect_options();
    format!(
        "postgres://postgres:postgres@{}:{}/{}",
        opts.get_host(),
        opts.get_port(),
        opts.get_database().unwrap_or("postgres")
    )
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
#[cfg_attr(
    target_os = "windows",
    ignore = "snapshot expectations only validated on unix-like platforms"
)]
async fn dblint_empty_database_snapshot(test_db: PgPool) {
    let url = get_database_url(&test_db);
    let output = run_dblint(&url, &[]);
    assert_snapshot!(output);
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
#[cfg_attr(
    target_os = "windows",
    ignore = "snapshot expectations only validated on unix-like platforms"
)]
async fn dblint_error_on_warnings_empty_database_snapshot(test_db: PgPool) {
    let url = get_database_url(&test_db);
    let output = run_dblint(&url, &["--error-on-warnings"]);
    assert_snapshot!(output);
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
#[cfg_attr(
    target_os = "windows",
    ignore = "snapshot expectations only validated on unix-like platforms"
)]
async fn dblint_detects_issues_snapshot(test_db: PgPool) {
    // Setup: create table without primary key (triggers noPrimaryKey rule)
    sqlx::raw_sql("CREATE TABLE test_no_pk (id int, name text)")
        .execute(&test_db)
        .await
        .expect("Failed to create test table");

    let url = get_database_url(&test_db);
    let output = run_dblint(&url, &[]);
    assert_snapshot!(output);
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
#[cfg_attr(
    target_os = "windows",
    ignore = "snapshot expectations only validated on unix-like platforms"
)]
async fn dblint_error_on_warnings_detects_issues_snapshot(test_db: PgPool) {
    // Setup: create duplicate non-unique indexes (triggers duplicateIndex warning)
    sqlx::raw_sql(
        "CREATE TABLE test_duplicate_idx (id int primary key, email text);
         CREATE INDEX idx_duplicate_a ON test_duplicate_idx (email);
         CREATE INDEX idx_duplicate_b ON test_duplicate_idx (email);",
    )
    .execute(&test_db)
    .await
    .expect("Failed to create test schema");

    let url = get_database_url(&test_db);
    let output = run_dblint(&url, &["--error-on-warnings"]);
    assert_snapshot!(output);
}

#[test]
#[cfg_attr(
    target_os = "windows",
    ignore = "snapshot expectations only validated on unix-like platforms"
)]
fn dblint_no_database_snapshot() {
    // Test that dblint completes gracefully when no database is configured
    let mut cmd = Command::cargo_bin(BIN).expect("binary not built");
    let output = cmd
        .args(["dblint", "--disable-db", "--log-level", "none"])
        .output()
        .expect("failed to run CLI");

    let normalized = normalize_output(
        output.status,
        &String::from_utf8_lossy(&output.stdout),
        &String::from_utf8_lossy(&output.stderr),
    );
    assert_snapshot!(normalized);
}

fn run_dblint(url: &str, args: &[&str]) -> String {
    let mut cmd = Command::cargo_bin(BIN).expect("binary not built");
    let mut full_args = vec!["dblint", "--connection-string", url, "--log-level", "none"];
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
