use pgt_query::protobuf::CaseExpr;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
};

pub(super) fn emit_case_expr(e: &mut EventEmitter, n: &CaseExpr) {
    e.group_start(GroupKind::CaseExpr);

    e.token(TokenKind::CASE_KW);

    // Optional test expression (for simple CASE)
    if let Some(ref arg) = n.arg {
        e.space();
        super::emit_node(arg, e);
    }

    // WHEN clauses
    for when_clause in &n.args {
        e.line(LineType::SoftOrSpace);
        super::emit_node(when_clause, e);
    }

    // ELSE clause
    if let Some(ref defresult) = n.defresult {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::ELSE_KW);
        e.space();
        super::emit_node(defresult, e);
    }

    e.line(LineType::SoftOrSpace);
    e.token(TokenKind::END_KW);

    e.group_end();
}
