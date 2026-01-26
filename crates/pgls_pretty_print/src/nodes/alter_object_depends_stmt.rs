use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind, LineType};
use pgls_query::protobuf::{AlterObjectDependsStmt, ObjectType};

pub(super) fn emit_alter_object_depends_stmt(e: &mut EventEmitter, n: &AlterObjectDependsStmt) {
    e.group_start(GroupKind::AlterObjectDependsStmt);

    e.token(TokenKind::ALTER_KW);
    e.space();

    // Object type
    match n.object_type() {
        ObjectType::ObjectFunction => e.token(TokenKind::FUNCTION_KW),
        ObjectType::ObjectProcedure => e.token(TokenKind::PROCEDURE_KW),
        ObjectType::ObjectRoutine => e.token(TokenKind::ROUTINE_KW),
        _ => e.token(TokenKind::IDENT("UNKNOWN".to_string())),
    }
    e.space();

    // Object name
    if let Some(ref object) = n.object {
        super::emit_node(object, e);
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
        e.token(TokenKind::IDENT(extname.sval.clone()));
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
