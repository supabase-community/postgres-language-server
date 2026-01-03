use tree_sitter::Language;
use tree_sitter::StreamingIterator;

fn main() {
    let sql = r#"select * from auth.users order REPLACED_TOKEN"#;

    let language: Language = pgls_treesitter_grammar::LANGUAGE.into();

    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&language)
        .expect("Setting Language failed.");

    let tree = parser
        .parse(sql.trim(), None)
        .expect("Failed to parse query.");

    let mut walker = tree.walk();

    walker.goto_first_child_for_byte(27);

    let byte_node = walker.node();

    let closest_before = byte_node.prev_sibling().map(|sib| {
        let mut curs = sib.walk();

        while curs.goto_last_child() {}

        curs.node()
    });

    println!(
        "closest_before {}",
        closest_before.map(|it| it.kind()).unwrap_or("nothing")
    );

    let query = tree_sitter::Query::new(
        &pgls_treesitter_grammar::LANGUAGE.into(),
        r#"
          (keyword_order) @tbl
        "#,
    )
    .expect("Invalid TS Query");

    let mut cursor = tree_sitter::QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), sql.as_bytes());

    let token = matches
        .next()
        .expect("invalid TS query for the SQL")
        .captures[0]
        .node
        .parent()
        .unwrap();

    println!("token kind is {}", token.kind());

    let mut lookahead = language.lookahead_iterator(token.parse_state()).unwrap();

    let tokens: Vec<&'static str> = lookahead.iter_names().collect();

    println!("{:#?}", tokens);
}
