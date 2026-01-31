use pgls_query::{NodeEnum, protobuf::DropdbStmt};

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
    nodes::node_list::emit_comma_separated_list,
};

pub(super) fn emit_dropdb_stmt(e: &mut EventEmitter, n: &DropdbStmt) {
    e.group_start(GroupKind::DropdbStmt);

    e.token(TokenKind::DROP_KW);
    e.space();
    e.token(TokenKind::DATABASE_KW);

    if n.missing_ok {
        e.space();
        e.token(TokenKind::IF_KW);
        e.space();
        e.token(TokenKind::EXISTS_KW);
    }

    if !n.dbname.is_empty() {
        e.space();
        e.token(TokenKind::IDENT(n.dbname.clone()));
    }

    // DROP DATABASE options like (FORCE)
    if !n.options.is_empty() {
        e.space();
        e.token(TokenKind::L_PAREN);
        emit_comma_separated_list(e, &n.options, |node, emitter| {
            if let Some(NodeEnum::DefElem(def)) = node.node.as_ref() {
                // Options like FORCE have just the name, no value
                emitter.token(TokenKind::IDENT(def.defname.to_uppercase()));
            }
        });
        e.token(TokenKind::R_PAREN);
    }

    e.token(TokenKind::SEMICOLON);
    e.group_end();
}
