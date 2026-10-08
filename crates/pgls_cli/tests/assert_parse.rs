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
