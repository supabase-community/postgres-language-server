use pgls_query::protobuf::{DropBehavior, TruncateStmt};

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
};

use super::node_list::emit_comma_separated_list;

pub(super) fn emit_truncate_stmt(e: &mut EventEmitter, n: &TruncateStmt) {
    e.group_start(GroupKind::TruncateStmt);

    e.token(TokenKind::TRUNCATE_KW);

    if !n.relations.is_empty() {
        e.line(LineType::SoftOrSpace);
        emit_comma_separated_list(e, &n.relations, super::emit_node);
    }

    // RESTART IDENTITY / CONTINUE IDENTITY
    if n.restart_seqs {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::RESTART_KW);
        e.space();
        e.token(TokenKind::IDENTITY_KW);
    }

    // CASCADE / RESTRICT
    match n.behavior() {
        DropBehavior::DropCascade => {
            e.line(LineType::SoftOrSpace);
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
