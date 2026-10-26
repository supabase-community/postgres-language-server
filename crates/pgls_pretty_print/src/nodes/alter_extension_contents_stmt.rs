use pgls_query::protobuf::AlterExtensionContentsStmt;

use super::prelude::*;

pub(super) fn emit_alter_extension_contents_stmt(
    e: &mut EventEmitter,
    n: &AlterExtensionContentsStmt,
) {
    e.group_start();

    e.token(TokenKind::ALTER_KW);
    e.space();
    e.token(TokenKind::EXTENSION_KW);
    e.space();

    if !n.extname.is_empty() {
        emit_identifier_maybe_quoted(e, &n.extname);
    }

    e.line(LineType::SoftOrSpace);

    // action: 1=ADD, -1=DROP
    if n.action == 1 {
        e.token(TokenKind::ADD_KW);
    } else {
        e.token(TokenKind::DROP_KW);
    }

    e.space();

    // Object type
    emit_keywords(e, object_type_keywords(n.objtype()).unwrap_or("OBJECT"));
    e.space();

    // Object name
    if let Some(ref object) = n.object {
        emit_node(e, object);
    }

    e.group_end();
}
