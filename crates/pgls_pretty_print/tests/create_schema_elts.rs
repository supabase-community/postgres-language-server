use pgls_pretty_print::{
    FormatConfig,
    emitter::EventEmitter,
    nodes::emit_node_enum,
    normalize::normalize_ast,
    renderer::{RenderConfig, Renderer},
};

/// `CREATE SCHEMA` takes a list of `schema_element`s. Per the PostgreSQL
/// grammar (`OptSchemaEltList: OptSchemaEltList schema_stmt`) the elements are
/// space-separated and must not carry their own terminating `;` — only the
/// outer statement does.
///
/// Regression test: the emitter used to emit each child statement's `;`,
/// producing invalid SQL like `create schema s1 create table a (f1 int);;`.
#[test]
fn create_schema_with_elements_emits_valid_sql() {
    let content = "
        CREATE SCHEMA s1
        CREATE TABLE a (f1 int)
        CREATE VIEW v AS SELECT f1 FROM a
        CREATE INDEX a_f1_idx ON a (f1)
        CREATE SEQUENCE seq
        CREATE TRIGGER trig BEFORE INSERT ON a EXECUTE FUNCTION no_func()
        GRANT SELECT ON TABLE a TO PUBLIC;
    ";

    let parsed = pgls_query::parse(content).expect("Failed to parse SQL");
    let mut ast = parsed.into_root().expect("No root node found");

    let mut emitter = EventEmitter::new(FormatConfig::default());
    emit_node_enum(&ast, &mut emitter);

    let mut output = String::new();
    let config = RenderConfig {
        max_line_length: 80,
        ..Default::default()
    };
    let mut renderer = Renderer::new(&mut output, config);
    renderer.render(emitter.events).expect("Failed to render");

    assert_eq!(output.matches(';').count(), 1, "{output}");

    let parsed_output = pgls_query::parse(&output).expect("Failed to parse formatted SQL");
    let mut parsed_ast = parsed_output.into_root().expect("No root node found");

    normalize_ast(&mut parsed_ast);
    normalize_ast(&mut ast);

    assert_eq!(ast, parsed_ast);
}
