use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind, LineType};
use pgls_query::protobuf::{AlterExtensionContentsStmt, ObjectType};

pub(super) fn emit_alter_extension_contents_stmt(
    e: &mut EventEmitter,
    n: &AlterExtensionContentsStmt,
) {
    e.group_start(GroupKind::AlterExtensionContentsStmt);

    e.token(TokenKind::ALTER_KW);
    e.space();
    e.token(TokenKind::EXTENSION_KW);
    e.space();

    if !n.extname.is_empty() {
        e.token(TokenKind::IDENT(n.extname.clone()));
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
    match n.objtype() {
        ObjectType::ObjectTable => e.token(TokenKind::TABLE_KW),
        ObjectType::ObjectFunction => e.token(TokenKind::FUNCTION_KW),
        ObjectType::ObjectType => e.token(TokenKind::TYPE_KW),
        ObjectType::ObjectOperator => e.token(TokenKind::OPERATOR_KW),
        _ => e.token(TokenKind::OBJECT_KW),
    }
    e.space();

    // Object name
    if let Some(ref object) = n.object {
        super::emit_node(object, e);
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
