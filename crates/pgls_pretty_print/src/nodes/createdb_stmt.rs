use pgls_query::protobuf::CreatedbStmt;

use super::prelude::*;

pub(super) fn emit_createdb_stmt(e: &mut EventEmitter, n: &CreatedbStmt) {
    e.group_start();

    e.token(TokenKind::CREATE_KW);
    e.space();
    e.token(TokenKind::DATABASE_KW);

    if !n.dbname.is_empty() {
        e.space();
        emit_identifier_maybe_quoted(e, &n.dbname);
    }

    // Emit database options (WITH CONNECTION LIMIT, ENCODING, etc.)
    if !n.options.is_empty() {
        e.space();
        e.token(TokenKind::WITH_KW);
        e.space();
        emit_space_separated_list(e, &n.options, |e, option| {
            emit_node(e, option);
            // `createdb_opt_name opt_equal DEFAULT` has no value
            // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/gram.y#L11414
            if let Some(NodeEnum::DefElem(def)) = option.node.as_ref()
                && def.arg.is_none()
            {
                e.space();
                e.token(TokenKind::IDENT("=".to_string()));
                e.space();
                e.token(TokenKind::DEFAULT_KW);
            }
        });
    }

    e.group_end();
}
