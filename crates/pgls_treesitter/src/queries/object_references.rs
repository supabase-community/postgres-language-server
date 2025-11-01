use std::sync::LazyLock;

use tree_sitter::Node;
use tree_sitter::StreamingIterator;

static PARTS_OF_REFERENCE_QUERY: LazyLock<tree_sitter::Query> = LazyLock::new(|| {
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
    (column_reference
        column_reference_1of1: (any_identifier) @tail
    )
    (column_reference
        column_reference_1of2: (any_identifier) @head
        column_reference_2of2: (any_identifier) @tail
    )
    (column_reference
        column_reference_1of3: (schema_identifier) @head
        column_reference_2of3: (table_identifier) @middle
        column_reference_3of3: (column_identifier) @tail
    )
    (table_reference
        (any_identifier) @tail
    )
    (table_reference
        (schema_identifier) @head
        (table_identifier) @tail
    )
    (type_reference
        (any_identifier) @tail
    )
    (type_reference
        (schema_identifier) @head
        (type_identifier) @tail
    )
    (function_reference
        (any_identifier) @tail
    )
    (function_reference
        (schema_identifier) @head
        (function_identifier) @tail
    )
"#;
    tree_sitter::Query::new(&pgls_treesitter_grammar::LANGUAGE.into(), QUERY_STR)
        .expect("Invalid TS Query")
});

pub fn parts_of_reference_query<'a>(
    node: Node<'a>,
    stmt: &'a str,
) -> Option<(Option<Node<'a>>, Option<Node<'a>>, Node<'a>)> {
    let mut cursor = tree_sitter::QueryCursor::new();
    let mut matches = cursor.matches(&PARTS_OF_REFERENCE_QUERY, node, stmt.as_bytes());

    if let Some(next) = matches.next() {
        if next.captures.len() == 1 {
            return Some((None, None, next.captures[0].node));
        };

        if next.captures.len() == 2 {
            return Some((None, Some(next.captures[0].node), next.captures[1].node));
        };

        if next.captures.len() == 3 {
            return Some((
                Some(next.captures[0].node),
                Some(next.captures[1].node),
                next.captures[2].node,
            ));
        };
    }

    None
}
