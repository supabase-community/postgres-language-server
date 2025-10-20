use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
};
use pgt_query::protobuf::{CoercionContext, CreateCastStmt};

pub(super) fn emit_create_cast_stmt(e: &mut EventEmitter, n: &CreateCastStmt) {
    e.group_start(GroupKind::CreateCastStmt);

    e.token(TokenKind::CREATE_KW);
    e.space();
    e.token(TokenKind::CAST_KW);
    e.space();
    e.token(TokenKind::L_PAREN);

    // Source type
    if let Some(ref source) = n.sourcetype {
        super::emit_type_name(e, source);
    }

    e.space();
    e.token(TokenKind::AS_KW);
    e.space();

    // Target type
    if let Some(ref target) = n.targettype {
        super::emit_type_name(e, target);
    }

    e.token(TokenKind::R_PAREN);
    e.line(LineType::SoftOrSpace);

    // WITH clause
    if let Some(ref func) = n.func {
        e.token(TokenKind::WITH_KW);
        e.space();
        e.token(TokenKind::FUNCTION_KW);
        e.space();
        super::emit_object_with_args(e, func);
    } else if n.inout {
        e.token(TokenKind::WITH_KW);
        e.space();
        e.token(TokenKind::IDENT("INOUT".to_string()));
    } else {
        e.token(TokenKind::WITHOUT_KW);
        e.space();
        e.token(TokenKind::FUNCTION_KW);
    }

    // Context: 0=IMPLICIT, 1=ASSIGNMENT, 2=EXPLICIT
    match n.context() {
        CoercionContext::CoercionImplicit => {
            e.line(LineType::SoftOrSpace);
            e.token(TokenKind::AS_KW);
            e.space();
            e.token(TokenKind::IDENT("IMPLICIT".to_string()));
        }
        CoercionContext::CoercionAssignment => {
            e.line(LineType::SoftOrSpace);
            e.token(TokenKind::AS_KW);
            e.space();
            e.token(TokenKind::IDENT("ASSIGNMENT".to_string()));
        }
        _ => {}
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
