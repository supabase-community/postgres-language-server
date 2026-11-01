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

    // Object name
    if let Some(ref object) = n.object {
        emit_node(e, object);
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
