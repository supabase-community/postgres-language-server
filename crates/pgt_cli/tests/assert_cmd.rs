use std::path::PathBuf;

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_cli_check_command() {
    let mut cmd = Command::cargo_bin("postgrestools").unwrap();

    let test_sql_path = PathBuf::from("tests/fixtures/test.sql");

    cmd.args(["check", test_sql_path.to_str().unwrap()])
        .assert()
        .failure()
        .stdout(predicate::str::contains("Found 1 error"));
}
