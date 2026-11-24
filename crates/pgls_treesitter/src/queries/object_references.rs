use std::sync::LazyLock;

use tree_sitter::Node;
use tree_sitter::StreamingIterator;

static PARTS_OF_REFERENCE_QUERY: LazyLock<Option<tree_sitter::Query>> = LazyLock::new(|| {
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
        // A query that doesn't compile fails the tests of this module.
        .ok()
});

pub fn parts_of_reference_query<'a>(
    node: Node<'a>,
    stmt: &'a str,
) -> Option<(Option<Node<'a>>, Option<Node<'a>>, Node<'a>)> {
    let mut cursor = tree_sitter::QueryCursor::new();
    let query = PARTS_OF_REFERENCE_QUERY.as_ref()?;
    let mut matches = cursor.matches(query, node, stmt.as_bytes());

    match matches.next()?.captures {
        [name] => Some((None, None, name.node)),
        [qualifier, name] => Some((None, Some(qualifier.node), name.node)),
        [schema, qualifier, name] => Some((Some(schema.node), Some(qualifier.node), name.node)),
        _ => None,
    }
}
