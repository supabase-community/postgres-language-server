use tree_sitter::{Node, Tree};

pub static SINGLE_TOKEN_RULES: &[&'static str] = &[
    "any_identifier",
    "column_identifier",
    "schema_identifier",
    "table_identifier",
    "function_identifier",
    "type_identifier",
    "type",
    "role_identifier",
    "policy_identifier",
    "object_reference",
    "table_reference",
    "column_reference",
    "function_reference",
    "type_reference",
    "composite_reference",
    "literal",
    "term",
    "parameter",
    "direction",
    "field",
    "bang",
    "op_other",
    "op_unary_other",
    "comment",
    "marginalia",
];

pub fn goto_node_at_position(tree: &Tree, position: usize) -> Option<Node<'_>> {
    let root = tree.root_node();

    if position >= root.end_byte() || position < root.start_byte() {
        return None;
    }

    let mut cursor = tree.root_node().walk();

    while cursor.goto_first_child_for_byte(position).is_some() {}

    Some(cursor.node())
}

pub fn goto_previous_leaf(node: Node<'_>) -> Option<Node<'_>> {
    let mut node_with_sibs = Some(node);

    while node_with_sibs
        .is_some_and(|node| node.kind() != "program" && node.prev_sibling().is_none())
    {
        node_with_sibs = node_with_sibs.unwrap().parent();
    }

    node_with_sibs.and_then(|node| {
        node.prev_sibling().map(|sib| {
            let mut cursor = sib.walk();
            while cursor.goto_last_child() {}
            cursor.node()
        })
    })
}

pub fn goto_closest_parent_clause(node: Node<'_>) -> Option<Node<'_>> {
    let mut parent = Some(node);

    while let Some(investigated) = parent {
        let kind = investigated.kind();

        let multiple_children = investigated.child_count() > 1;

        let single_child_uncompleted =
            investigated.child_count() == 1 && investigated.child_by_field_name("end").is_some();

        let explicit_skip = SINGLE_TOKEN_RULES.contains(&kind);

        let is_error = investigated.kind() == "ERROR";

        if !explicit_skip {
            if multiple_children || single_child_uncompleted || is_error {
                return Some(investigated);
            }
        }

        parent = investigated.parent();
    }

    return None;
}
