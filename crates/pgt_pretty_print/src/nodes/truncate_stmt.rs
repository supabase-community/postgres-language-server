use pgt_query::protobuf::{DropBehavior, TruncateStmt};

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

use super::node_list::emit_comma_separated_list;

pub(super) fn emit_truncate_stmt(e: &mut EventEmitter, n: &TruncateStmt) {
    e.group_start(GroupKind::TruncateStmt);

    e.token(TokenKind::TRUNCATE_KW);

    if !n.relations.is_empty() {
        e.space();
        emit_comma_separated_list(e, &n.relations, super::emit_node);
    }

    // RESTART IDENTITY / CONTINUE IDENTITY
    if n.restart_seqs {
        e.space();
        e.token(TokenKind::RESTART_KW);
        e.space();
        e.token(TokenKind::IDENTITY_KW);
    }

    // CASCADE / RESTRICT
    match n.behavior() {
        DropBehavior::DropCascade => {
            e.space();
            e.token(TokenKind::CASCADE_KW);
        }
        DropBehavior::DropRestrict => {
            // RESTRICT is default, usually not emitted
        }
        DropBehavior::Undefined => {
            // Undefined behavior, don't emit anything
        }
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
