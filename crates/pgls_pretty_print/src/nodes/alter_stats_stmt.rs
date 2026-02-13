use super::node_list::emit_dot_separated_list;
use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
};
use pgls_query::protobuf::AlterStatsStmt;

pub(super) fn emit_alter_stats_stmt(e: &mut EventEmitter, n: &AlterStatsStmt) {
    e.group_start(GroupKind::AlterStatsStmt);

    e.token(TokenKind::ALTER_KW);
    e.space();
    e.token(TokenKind::STATISTICS_KW);
    e.space();

    // IF EXISTS
    if n.missing_ok {
        e.token(TokenKind::IF_KW);
        e.space();
        e.token(TokenKind::EXISTS_KW);
        e.space();
    }

    // Statistics name
    emit_dot_separated_list(e, &n.defnames);

    // SET STATISTICS target
    if let Some(ref target) = n.stxstattarget {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::SET_KW);
        e.space();
        e.token(TokenKind::STATISTICS_KW);
        e.space();
        super::emit_node(target, e);
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
