use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
    nodes::{node_list::emit_comma_separated_list, node_list::emit_dot_separated_list},
};
use pgls_query::{NodeEnum, protobuf::CreateStatsStmt};

pub(super) fn emit_create_stats_stmt(e: &mut EventEmitter, n: &CreateStatsStmt) {
    e.group_start(GroupKind::CreateStatsStmt);

    e.token(TokenKind::CREATE_KW);
    e.space();
    e.token(TokenKind::IDENT("statistics".to_string()));

    if n.if_not_exists {
        e.space();
        e.token(TokenKind::IF_KW);
        e.space();
        e.token(TokenKind::NOT_KW);
        e.space();
        e.token(TokenKind::EXISTS_KW);
    }

    if !n.defnames.is_empty() {
        e.space();
        emit_dot_separated_list(e, &n.defnames);
    }

    // Statistics types (e.g., ndistinct, dependencies)
    if !n.stat_types.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::L_PAREN);
        emit_comma_separated_list(e, &n.stat_types, |node, e| {
            if let Some(NodeEnum::String(s)) = &node.node {
                e.token(TokenKind::IDENT(s.sval.clone()));
            }
        });
        e.token(TokenKind::R_PAREN);
    }

    e.line(LineType::SoftOrSpace);
    e.token(TokenKind::ON_KW);
    e.space();

    // Column expressions or names
    if !n.exprs.is_empty() {
        emit_comma_separated_list(e, &n.exprs, |node, e| {
            if let Some(NodeEnum::StatsElem(stats_elem)) = &node.node {
                super::emit_stats_elem(e, stats_elem);
            }
        });
    }

    e.line(LineType::SoftOrSpace);
    e.token(TokenKind::FROM_KW);
    e.space();

    // Relations (tables or subselects)
    emit_comma_separated_list(e, &n.relations, super::emit_node);

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
