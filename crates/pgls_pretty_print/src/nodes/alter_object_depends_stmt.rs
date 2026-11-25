use pgls_query::protobuf::AlterObjectDependsStmt;

use super::prelude::*;

pub(super) fn emit_alter_object_depends_stmt(e: &mut EventEmitter, n: &AlterObjectDependsStmt) {
    e.group_start();

    e.token(TokenKind::ALTER_KW);
    e.space();

    // Object type
    match object_type_keywords(n.object_type()) {
        Some(keywords) => emit_keywords(e, keywords),
        None => e.token(TokenKind::IDENT("UNKNOWN".to_string())),
    }
    e.space();

    // An index or materialized view is the relation. A trigger is `name ON relation`, with the name
    // as the only item of the object.
    // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/gram.y#L10019
    match (&n.object, &n.relation) {
        (Some(object), Some(relation)) => {
            emit_any_name(e, object);
            e.space();
            e.token(TokenKind::ON_KW);
            e.space();
            super::emit_range_var(e, relation);
        }
        (Some(object), None) => emit_node(e, object),
        (None, Some(relation)) => super::emit_range_var(e, relation),
        (None, None) => {}
    }

    e.line(LineType::SoftOrSpace);

    if n.remove {
        e.token(TokenKind::NO_KW);
        e.space();
    }

    e.token(TokenKind::DEPENDS_KW);
    e.space();
    e.token(TokenKind::ON_KW);
    e.space();
    e.token(TokenKind::EXTENSION_KW);

    if let Some(ref extname) = n.extname {
        e.space();
        emit_identifier_maybe_quoted(e, &extname.sval);
    }

    e.group_end();
}
