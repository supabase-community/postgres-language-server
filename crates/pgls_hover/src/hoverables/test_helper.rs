#[cfg(test)]
use pgls_test_utils::QueryWithCursorPosition;
#[cfg(test)]
use pgls_treesitter::TreesitterContext;

#[cfg(test)]
pub(crate) fn create_test_context(query: QueryWithCursorPosition) -> TreesitterContext<'static> {
    use pgls_text_size::TextSize;
    use pgls_treesitter::TreeSitterContextParams;

    let (pos, sql) = query.get_text_and_position();

    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&pgls_treesitter_grammar::LANGUAGE.into())
        .unwrap();
    let tree = parser.parse(sql.clone(), None).unwrap();

    // Leak some stuff so test setup is easier
    let leaked_tree: &'static tree_sitter::Tree = Box::leak(Box::new(tree));
    let leaked_sql: &'static String = Box::leak(Box::new(sql));

    let position = TextSize::new(pos.try_into().unwrap());

    pgls_treesitter::context::TreesitterContext::new(TreeSitterContextParams {
        position,
        text: leaked_sql,
        tree: leaked_tree,
    })
}
