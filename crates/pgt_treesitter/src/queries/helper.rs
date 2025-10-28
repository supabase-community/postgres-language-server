use tree_sitter::Node;

pub(crate) static OBJECT_REFERENCE_QUERY: LazyLock<tree_sitter::Query> = LazyLock::new(|| {
    static QUERY_STR: &str = r#"
        (object_reference
          object_reference_1of1: (any_identifier) @tail
        )
        (object_reference
          object_reference_1of2: (any_identifier) @head
          object_reference_2of2: (any_identifier) @tail
        )
        (object_reference
          object_reference_1of1: (any_identifier) @head
          object_reference_1of2: (any_identifier) @middle
          object_reference_1of3: (any_identifier) @tail
        )
    )
"#;
    tree_sitter::Query::new(&pgt_treesitter_grammar::LANGUAGE.into(), QUERY_STR)
        .expect("Invalid TS Query")
});

pub(crate) fn object_reference_query<'a>(
    node: Node<'a>,
    stmt: &'a str,
) -> Option<(Option<Node<'a>>, Option<Node<'a>>, Node<'a>)> {
    let mut cursor = tree_sitter::QueryCursor::new();
    let matches = cursor.matches(&TS_QUERY, root_node, stmt.as_bytes());

    assert!(
        matches.len() <= 1,
        "Please pass a single `object_reference` node into the `object_reference_query`!"
    );

    if matches[0].len() == 0 {
        None
    } else if matches[0].captures.len() == 1 {
        Some((None, None, m.captures[0].node))
    } else if matches[0].captures.len() == 2 {
        Some((None, m.captures[0].node, m.captures[1].node))
    } else if matches[0].captures.len() == 3 {
        Some((
            Some(m.captures[0].node),
            Some(m.captures[1].node),
            m.captures[2].node,
        ))
    } else {
        None
    }
}
