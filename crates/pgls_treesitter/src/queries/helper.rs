use std::sync::LazyLock;

use tree_sitter::Node;
use tree_sitter::StreamingIterator;

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
        object_reference_1of3: (any_identifier) @head
        object_reference_2of3: (any_identifier) @middle
        object_reference_3of3: (any_identifier) @tail
    )
"#;
    tree_sitter::Query::new(&pgls_treesitter_grammar::LANGUAGE.into(), QUERY_STR)
        .expect("Invalid TS Query")
});

pub(crate) fn object_reference_query<'a>(
    node: Node<'a>,
    stmt: &'a str,
) -> Option<(Option<Node<'a>>, Option<Node<'a>>, Node<'a>)> {
    let mut cursor = tree_sitter::QueryCursor::new();
    let mut matches = cursor.matches(&OBJECT_REFERENCE_QUERY, node, stmt.as_bytes());

    if let Some(next) = matches.next() {
        if next.captures.len() == 1 {
            Some((None, None, next.captures[0].node))
        } else if next.captures.len() == 2 {
            Some((None, Some(next.captures[0].node), next.captures[1].node))
        } else if next.captures.len() == 3 {
            Some((
                Some(next.captures[0].node),
                Some(next.captures[1].node),
                next.captures[2].node,
            ))
        } else {
            None
        }
    } else {
        None
    }
}
