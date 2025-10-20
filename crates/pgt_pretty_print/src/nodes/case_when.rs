use pgt_query::protobuf::CaseWhen;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

pub(super) fn emit_case_when(e: &mut EventEmitter, n: &CaseWhen) {
    e.group_start(GroupKind::CaseWhen);

    e.token(TokenKind::WHEN_KW);

    if let Some(ref expr) = n.expr {
        e.space();
        super::emit_node(expr, e);
    }

    e.space();
    e.token(TokenKind::THEN_KW);

    if let Some(ref result) = n.result {
        e.space();
        super::emit_node(result, e);
    }

    e.group_end();
}
