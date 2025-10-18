use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind};
use pgt_query::protobuf::{AlterExtensionContentsStmt, ObjectType};

pub(super) fn emit_alter_extension_contents_stmt(
    e: &mut EventEmitter,
    n: &AlterExtensionContentsStmt,
) {
    e.group_start(GroupKind::AlterExtensionContentsStmt);

    e.token(TokenKind::ALTER_KW);
    e.space();
    e.token(TokenKind::IDENT("EXTENSION".to_string()));
    e.space();

    if !n.extname.is_empty() {
        e.token(TokenKind::IDENT(n.extname.clone()));
    }

    e.space();

    // action: 1=ADD, -1=DROP
    if n.action == 1 {
        e.token(TokenKind::ADD_KW);
    } else {
        e.token(TokenKind::DROP_KW);
    }

    e.space();

    // Object type
    let object_type_str = match ObjectType::try_from(n.objtype) {
        Ok(ObjectType::ObjectTable) => "TABLE",
        Ok(ObjectType::ObjectFunction) => "FUNCTION",
        Ok(ObjectType::ObjectType) => "TYPE",
        Ok(ObjectType::ObjectOperator) => "OPERATOR",
        _ => "OBJECT",
    };
    e.token(TokenKind::IDENT(object_type_str.to_string()));
    e.space();

    // Object name
    if let Some(ref object) = n.object {
        super::emit_node(object, e);
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
