#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "a test fails on unexpected data; clippy exempts `#[test]` functions, not their helpers"
)]

use assert_cmd::cargo_bin_cmd;
use prost::Message;

/// Run `postgres-language-server parse` on a SQL string and decode the
/// protobuf `ParseResult` from standard output.
fn run_parse(sql: &str) -> pgls_query::protobuf::ParseResult {
    let output = cargo_bin_cmd!("postgres-language-server")
        .args(["parse"])
        .write_stdin(sql)
        .output()
        .expect("run postgres-language-server parse");

    assert!(
        output.status.success(),
        "parse failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    pgls_query::protobuf::ParseResult::decode(output.stdout.as_slice())
        .expect("decode ParseResult protobuf")
}

#[test]
fn parse_emits_a_decodable_parse_result() {
    let parsed = run_parse("SELECT id FROM example_table;\n");
    assert_eq!(parsed.stmts.len(), 1);
}

#[test]
fn parse_keeps_named_parameter_locations_aligned() {
    // psql named parameters become parser-compatible placeholders of identical
    // byte length, so every location of the tree still matches the input.
    let sql = "SELECT t.id\nFROM :raw_data.documents AS t\nWHERE t.agency = :'agen_code';\n";
    let parsed = run_parse(sql);
    assert_eq!(parsed.stmts.len(), 1);

    let raw = &parsed.stmts[0];
    assert_eq!(raw.stmt_location, 0);
    // The statement must end exactly where the input's semicolon sits: named
    // parameters are substituted by placeholders of identical byte length, so
    // tree locations stay aligned with the original bytes.
    assert_eq!(sql.as_bytes()[raw.stmt_len as usize], b';');
}

/// The outcome of `parse` with `args` in `cwd` on `stdin`: whether it succeeded, and its stderr.
struct ParseRun {
    success: bool,
    stderr: String,
}

impl ParseRun {
    fn new(cwd: &std::path::Path, args: &[&str], stdin: &str) -> Self {
        let output = cargo_bin_cmd!("postgres-language-server")
            .current_dir(cwd)
            .arg("parse")
            .args(args)
            .write_stdin(stdin)
            .output()
            .expect("run postgres-language-server parse");
        Self {
            success: output.status.success(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        }
    }
}

#[test]
fn parse_reads_templates_like_check() {
    let root = tempfile::tempdir().unwrap();
    let queries = root.path().join("queries");
    std::fs::create_dir_all(&queries).unwrap();
    std::fs::write(
        queries.join("postgres-language-server.jsonc"),
        r#"{ "templating": { "syntax": "go-printf" } }"#,
    )
    .unwrap();
    let template = "select * from %s.users where id = %d;\n";
    std::fs::write(queries.join("users.sql"), template).unwrap();
    let file = queries.join("users.sql");
    let file = file.to_str().unwrap();

    // The configuration of the folder of the file applies.
    let run = ParseRun::new(root.path(), &["--file", file], "");
    assert!(run.success, "{}", run.stderr);
    // `--template-syntax none` switches it off.
    let run = ParseRun::new(
        root.path(),
        &["--template-syntax", "none", "--file", file],
        "",
    );
    assert!(!run.success);
    // Standard input uses the configuration of the working directory, or the argument.
    assert!(!ParseRun::new(root.path(), &[], template).success);
    let run = ParseRun::new(root.path(), &["--template-syntax", "go-printf"], template);
    assert!(run.success, "{}", run.stderr);
    let config = queries.join("postgres-language-server.jsonc");
    let run = ParseRun::new(
        root.path(),
        &["--config-path", config.to_str().unwrap()],
        template,
    );
    assert!(run.success, "{}", run.stderr);

    // A directive that can't be read says why.
    let run = ParseRun::new(
        root.path(),
        &["--template-syntax", "rust-fmt"],
        "select * from t {alias};\n",
    );
    assert!(!run.success);
    assert!(
        run.stderr
            .contains("A template placeholder directly after `t` can't be read"),
        "{}",
        run.stderr
    );
}
