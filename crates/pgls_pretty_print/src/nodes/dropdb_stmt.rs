use pgls_query::protobuf::DropdbStmt;

use super::prelude::*;

pub(super) fn emit_dropdb_stmt(e: &mut EventEmitter, n: &DropdbStmt) {
    e.group_start();

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
        emit_identifier_maybe_quoted(e, &n.dbname);
    }

    // DROP DATABASE options like (FORCE)
    if !n.options.is_empty() {
        e.space();
        emit_parenthesized_list(e, &n.options, |emitter, node| {
            if let Some(NodeEnum::DefElem(def)) = node.node.as_ref() {
                // Options like FORCE have just the name, no value
                emit_keyword(emitter, &def.defname);
            }
        });
    }

    e.group_end();
}
