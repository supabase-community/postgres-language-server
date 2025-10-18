use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind};
use pgt_query::protobuf::{AlterObjectDependsStmt, ObjectType};

pub(super) fn emit_alter_object_depends_stmt(e: &mut EventEmitter, n: &AlterObjectDependsStmt) {
    e.group_start(GroupKind::AlterObjectDependsStmt);

    e.token(TokenKind::ALTER_KW);
    e.space();

    // Object type
    let object_type_str = match ObjectType::try_from(n.object_type) {
        Ok(ObjectType::ObjectFunction) => "FUNCTION",
        Ok(ObjectType::ObjectProcedure) => "PROCEDURE",
        Ok(ObjectType::ObjectRoutine) => "ROUTINE",
        _ => "UNKNOWN",
    };
    e.token(TokenKind::IDENT(object_type_str.to_string()));
    e.space();

    // Object name
    if let Some(ref object) = n.object {
        super::emit_node(object, e);
    }

    e.space();

    if n.remove {
        e.token(TokenKind::IDENT("NO".to_string()));
        e.space();
    }

    e.token(TokenKind::IDENT("DEPENDS".to_string()));
    e.space();
    e.token(TokenKind::ON_KW);
    e.space();
    e.token(TokenKind::IDENT("EXTENSION".to_string()));

    if let Some(ref extname) = n.extname {
        e.space();
        e.token(TokenKind::IDENT(extname.sval.clone()));
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
