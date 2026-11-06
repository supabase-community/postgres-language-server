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

#[test]
fn template_parse_dollar_values_match_native_postgresql_without_configuration() {
    let source = "INSERT INTO events VALUES ($1, $2, $123, $1);";
    let native = pgls_query::parse(source).unwrap().protobuf;
    assert_eq!(run_parse(source), native);
    let output = cargo_bin_cmd!("postgres-language-server")
        .args(["parse", "--template-syntax", "dollar-values"])
        .write_stdin(source)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let explicit = pgls_query::protobuf::ParseResult::decode(output.stdout.as_slice()).unwrap();
    assert_eq!(explicit, native);
}

#[test]
fn template_parse_accepts_mixed_modes_from_an_array_configuration() {
    let temp = tempfile::tempdir().unwrap();
    let config = temp.path().join("postgres-language-server.jsonc");
    std::fs::write(
        &config,
        r#"{"templating":{"syntax":["go-printf","rust-fmt","dollar-values"]}}"#,
    )
    .unwrap();
    let source = "SELECT {value}, $2 FROM %s.items WHERE id=$1;";
    let output = cargo_bin_cmd!("postgres-language-server")
        .arg("parse")
        .arg("--config-path")
        .arg(config)
        .write_stdin(source)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let parsed = pgls_query::protobuf::ParseResult::decode(output.stdout.as_slice()).unwrap();
    assert_eq!(parsed.stmts.len(), 1);
    assert_eq!(source.as_bytes()[parsed.stmts[0].stmt_len as usize], b';');
}

#[test]
fn template_parse_reports_invalid_fixed_sql_around_a_placeholder() {
    cargo_bin_cmd!("postgres-language-server")
        .args(["parse", "--template-syntax", "go-printf,dollar-values"])
        .write_stdin("SELECT * FROM %s WHERE = $1;")
        .assert()
        .failure();
}
