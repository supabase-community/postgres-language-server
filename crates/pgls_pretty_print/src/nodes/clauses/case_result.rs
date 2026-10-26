//! The result of a CASE branch, after THEN or ELSE.

use crate::nodes::prelude::*;

/// Emits a branch result, `then <result>` or `else <result>`.
///
/// In expanded layout a CASE aligns its branches with the column it starts at. A CASE nested as a
/// result would start after `then `, so every level would move the next one right by the keyword
/// and two indents, and a few levels run past the line width. A nested CASE is therefore indented
/// from the line of its THEN or ELSE instead: its branches one level in, `end` under `then`.
pub(crate) fn emit_case_result(e: &mut EventEmitter, result: &pgls_query::protobuf::Node) {
    emit_node_with(e, result, |e, node| match node {
        pgls_query::NodeEnum::CaseExpr(case) => {
            crate::nodes::emit_case_expr_as(e, case, crate::nodes::BranchPlacement::Indented)
        }
        node => emit_node_enum(e, node),
    });
}
