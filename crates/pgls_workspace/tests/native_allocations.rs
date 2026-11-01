//! The memory tests count what tree-sitter allocates in C.

mod common;

use common::{count_native_allocations, current};

#[test]
fn tree_sitter_trees_are_counted() {
    count_native_allocations();
    let sql = "select id, name from users where id = 1;\n".repeat(2_000);
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&pgls_treesitter_grammar::LANGUAGE.into())
        .unwrap();

    let before = current();
    let tree = parser.parse(&sql, None).unwrap();
    let with_tree = current();
    // A tree has a node per token, which takes more than a byte of the text.
    assert!(with_tree - before > sql.len(), "{before} {with_tree}");

    drop(tree);
    drop(parser);
    assert!(current() < with_tree);
}
