//! An item of `GROUP BY` or of `GROUPING SETS (...)`.

use crate::nodes::prelude::*;

/// Emits an item of `GROUP BY` or `GROUPING SETS (...)`. There, `cube(a)` and `rollup(a)` are
/// grouping sets, so a call of a function with that name keeps its parentheses: `(cube(a))`.
/// https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/gram.y#L13480
pub(crate) fn emit_group_by_item(e: &mut EventEmitter, node: &Node) {
    let reads_as_grouping_set = matches!(
        &node.node,
        Some(NodeEnum::FuncCall(call)) if matches!(
            call.funcname.as_slice(),
            [Node { node: Some(NodeEnum::String(name)) }] if matches!(name.sval.as_str(), "cube" | "rollup")
        )
    );
    if reads_as_grouping_set {
        e.token(TokenKind::L_PAREN);
        emit_node(e, node);
        e.token(TokenKind::R_PAREN);
    } else {
        emit_node(e, node);
    }
}
