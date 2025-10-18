use pgt_query::protobuf::CreateSchemaStmt;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

pub(super) fn emit_create_schema_stmt(e: &mut EventEmitter, n: &CreateSchemaStmt) {
    e.group_start(GroupKind::CreateSchemaStmt);

    e.token(TokenKind::CREATE_KW);
    e.space();
    e.token(TokenKind::SCHEMA_KW);

    if n.if_not_exists {
        e.space();
        e.token(TokenKind::IF_KW);
        e.space();
        e.token(TokenKind::NOT_KW);
        e.space();
        e.token(TokenKind::EXISTS_KW);
    }

    if !n.schemaname.is_empty() {
        e.space();
        super::emit_identifier(e, &n.schemaname);
    }

    // AUTHORIZATION clause
    if let Some(ref authrole) = n.authrole {
        e.space();
        e.token(TokenKind::AUTHORIZATION_KW);
        e.space();
        super::emit_role_spec(e, authrole);
    }

    // Schema elements (nested CREATE statements)
    if !n.schema_elts.is_empty() {
        e.space();
        for (i, elt) in n.schema_elts.iter().enumerate() {
            if i > 0 {
                e.space();
            }
            super::emit_node(elt, e);
        }
    }

    e.token(TokenKind::SEMICOLON);
    e.group_end();
}
