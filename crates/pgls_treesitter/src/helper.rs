use pgls_treesitter_grammar::{Field, Kind};
use tree_sitter::{Node, Tree};

pub static SINGLE_TOKEN_RULES: &[Kind] = &[
    Kind::ANY_IDENTIFIER,
    Kind::COLUMN_IDENTIFIER,
    Kind::SCHEMA_IDENTIFIER,
    Kind::TABLE_IDENTIFIER,
    Kind::FUNCTION_IDENTIFIER,
    Kind::TYPE_IDENTIFIER,
    Kind::TYPE,
    Kind::ROLE_IDENTIFIER,
    Kind::POLICY_IDENTIFIER,
    Kind::OBJECT_REFERENCE,
    Kind::TABLE_REFERENCE,
    Kind::COLUMN_REFERENCE,
    Kind::FUNCTION_REFERENCE,
    Kind::TYPE_REFERENCE,
    Kind::LITERAL,
    Kind::TERM,
    Kind::PARAMETER,
    Kind::DIRECTION,
    Kind::FIELD,
    Kind::BANG,
    Kind::OP_OTHER,
    Kind::OP_UNARY_OTHER,
    Kind::COMMENT,
    Kind::MARGINALIA,
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

    while let Some(node) = node_with_sibs
        && Kind::of(node) != Kind::PROGRAM
        && node.prev_sibling().is_none()
    {
        node_with_sibs = node.parent();
    }

    node_with_sibs.and_then(|node| {
        node.prev_sibling().map(|sib| {
            let mut cursor = sib.walk();
            while cursor.goto_last_child() {}
            cursor.node()
        })
    })
}

pub fn goto_closest_unfinished_parent_clause(node: Node<'_>) -> Option<Node<'_>> {
    let mut parent = Some(node);

    while let Some(investigated) = parent {
        let kind = Kind::of(investigated);

        // The top level node for all possible trees.
        if kind == Kind::PROGRAM {
            break;
        }

        let explicit_skip = SINGLE_TOKEN_RULES.contains(&kind);
        let is_parent = investigated.child_count() > 0;
        let is_finished = investigated.child_by_field_id(Field::END.id()).is_some();

        if !explicit_skip && is_parent && !is_finished {
            return Some(investigated);
        }

        parent = investigated.parent();
    }

    None
}

/// Whether the node before `node` is complete: a single token, or a node with its `end` whose last
/// child is complete too, down to a leaf. A loop over the last children, since tree-sitter doesn't
/// bound the depth of its trees.
pub fn previous_sibling_completed(node: tree_sitter::Node) -> bool {
    let Some(prev) = node.prev_sibling() else {
        return true;
    };

    let mut cursor = prev.walk();
    loop {
        let current = cursor.node();
        let explicit_skip = SINGLE_TOKEN_RULES.contains(&Kind::of(current));
        let is_parent = current.child_count() > 0;
        let is_finished = current.child_by_field_id(Field::END.id()).is_some();

        if explicit_skip || !is_parent {
            return true;
        }

        if !is_finished {
            return false;
        }

        if !cursor.goto_last_child() {
            return true;
        }
    }
}
