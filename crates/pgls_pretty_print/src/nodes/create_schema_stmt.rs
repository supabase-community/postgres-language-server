use pgls_query::{
    NodeEnum,
    protobuf::{CreateSchemaStmt, Node},
};

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

use super::node_list::emit_space_separated_list;
use crate::emitter::LineType;

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

    // Schema elements (nested CREATE statements).
    //
    // Per the PostgreSQL grammar (`OptSchemaEltList: OptSchemaEltList schema_stmt`),
    // elements are space-separated and the child statements must not carry their
    // own terminating `;` — only the outer `CREATE SCHEMA` does.
    if !n.schema_elts.is_empty() {
        // Soft line so the renderer can wrap when the schema name and the
        // first element don't fit on one line.
        e.line(LineType::SoftOrSpace);
        emit_space_separated_list(e, &n.schema_elts, emit_schema_element);
    }

    e.token(TokenKind::SEMICOLON);
    e.group_end();
}

fn emit_schema_element(node: &Node, e: &mut EventEmitter) {
    super::emit_node_with(node, e, |node, e| match node {
        NodeEnum::CreateStmt(n) => super::emit_create_stmt_no_semicolon(e, n),
        NodeEnum::ViewStmt(n) => super::emit_view_stmt_no_semicolon(e, n),
        NodeEnum::IndexStmt(n) => super::emit_index_stmt_no_semicolon(e, n),
        NodeEnum::CreateSeqStmt(n) => super::emit_create_seq_stmt_no_semicolon(e, n),
        NodeEnum::CreateTrigStmt(n) => super::emit_create_trig_stmt_no_semicolon(e, n),
        NodeEnum::GrantStmt(n) => super::emit_grant_stmt_no_semicolon(e, n),
        _ => super::emit_node_enum(node, e),
    });
}
