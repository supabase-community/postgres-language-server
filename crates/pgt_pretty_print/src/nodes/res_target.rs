use pgt_query::protobuf::ResTarget;

use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind};

use super::emit_identifier;
use super::emit_node;

pub(super) fn emit_res_target(e: &mut EventEmitter, n: &ResTarget) {
    e.group_start(GroupKind::ResTarget);

    if let Some(ref val) = n.val {
        emit_node(val, e);

        if !n.name.is_empty() {
            e.space();
            e.token(TokenKind::AS_KW);
            e.space();
            emit_identifier(e, &n.name);
        }
    }

    e.group_end();
}

pub(super) fn emit_set_clause(e: &mut EventEmitter, n: &ResTarget) {
    e.group_start(GroupKind::ResTarget);

    if !n.name.is_empty() {
        emit_column_name_with_indirection(e, n);

        if let Some(ref val) = n.val {
            e.space();
            e.token(TokenKind::IDENT("=".to_string()));
            e.space();
            emit_node(val, e);
        }
    }

    e.group_end();
}

pub(super) fn emit_column_name_with_indirection(e: &mut EventEmitter, n: &ResTarget) {
    if n.name.is_empty() {
        return;
    }

    e.token(TokenKind::IDENT(n.name.clone()));

    for i in &n.indirection {
        match &i.node {
            // Field selection - emit dot before the field name
            Some(pgt_query::NodeEnum::String(s)) => {
                e.token(TokenKind::DOT);
                super::emit_string_identifier(e, s);
            }
            Some(n) => super::emit_node_enum(n, e),
            None => {}
        }
    }
}

// Emit column name only (for INSERT column list)
pub(super) fn emit_column_name(e: &mut EventEmitter, n: &ResTarget) {
    e.group_start(GroupKind::ResTarget);
    emit_column_name_with_indirection(e, n);
    e.group_end();
}
