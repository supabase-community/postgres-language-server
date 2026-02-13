use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
};
use pgls_query::{NodeEnum, protobuf::CreateExtensionStmt};

pub(super) fn emit_create_extension_stmt(e: &mut EventEmitter, n: &CreateExtensionStmt) {
    e.group_start(GroupKind::CreateExtensionStmt);

    e.token(TokenKind::CREATE_KW);
    e.space();
    e.token(TokenKind::EXTENSION_KW);
    e.space();

    if n.if_not_exists {
        e.token(TokenKind::IF_KW);
        e.space();
        e.token(TokenKind::NOT_KW);
        e.space();
        e.token(TokenKind::EXISTS_KW);
        e.space();
    }

    e.token(TokenKind::IDENT(n.extname.clone()));

    if !n.options.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::WITH_KW);

        for opt in &n.options {
            let def = assert_node_variant!(DefElem, opt);
            e.space();
            emit_extension_option(e, def);
        }
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}

/// Emit a CREATE EXTENSION option.
/// Options use keyword syntax (no equals sign):
///   SCHEMA schema_name
///   VERSION 'version'
///   CASCADE
fn emit_extension_option(e: &mut EventEmitter, def: &pgls_query::protobuf::DefElem) {
    match def.defname.as_str() {
        "schema" => {
            e.token(TokenKind::SCHEMA_KW);
            if let Some(ref arg) = def.arg {
                e.space();
                // Schema name is a String node - emit_node dispatches to
                // emit_string which uses emit_identifier_maybe_quoted
                super::emit_node(arg, e);
            }
        }
        "new_version" => {
            e.token(TokenKind::VERSION_KW);
            if let Some(ref arg) = def.arg {
                e.space();
                // Version must be a quoted string literal
                if let Some(NodeEnum::String(s)) = &arg.node {
                    super::emit_string_literal(e, s);
                } else {
                    super::emit_node(arg, e);
                }
            }
        }
        "cascade" => {
            e.token(TokenKind::CASCADE_KW);
        }
        _ => {
            // Fallback for unknown options
            super::string::emit_keyword(e, &def.defname);
            if let Some(ref arg) = def.arg {
                e.space();
                super::emit_node(arg, e);
            }
        }
    }
}
