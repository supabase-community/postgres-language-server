use pgt_query::protobuf::VacuumStmt;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

use super::node_list::emit_comma_separated_list;

pub(super) fn emit_vacuum_stmt(e: &mut EventEmitter, n: &VacuumStmt) {
    e.group_start(GroupKind::VacuumStmt);

    if n.is_vacuumcmd {
        e.token(TokenKind::VACUUM_KW);
    } else {
        e.token(TokenKind::ANALYZE_KW);
    }

    // Options (TODO: parse options list properly)
    // For now, just skip options

    // Relations to vacuum/analyze
    if !n.rels.is_empty() {
        e.space();
        emit_comma_separated_list(e, &n.rels, super::emit_node);
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
