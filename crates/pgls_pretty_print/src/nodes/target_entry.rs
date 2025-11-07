use pgls_query::protobuf::TargetEntry;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

pub(super) fn emit_target_entry(e: &mut EventEmitter, n: &TargetEntry) {
    e.group_start(GroupKind::TargetEntry);

    if let Some(ref expr) = n.expr {
        super::emit_node(expr, e);
    }

    if !n.resname.is_empty() {
        e.space();
        e.token(TokenKind::AS_KW);
        e.space();
        super::emit_identifier_maybe_quoted(e, &n.resname);
    }

    e.group_end();
}
