use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind, LineType};
use pgls_query::protobuf::{AlterFunctionStmt, ObjectType};

use super::object_with_args::emit_object_with_args;

pub(super) fn emit_alter_function_stmt(e: &mut EventEmitter, n: &AlterFunctionStmt) {
    e.group_start(GroupKind::AlterFunctionStmt);

    e.token(TokenKind::ALTER_KW);
    e.space();

    match n.objtype() {
        ObjectType::ObjectProcedure => e.token(TokenKind::PROCEDURE_KW),
        ObjectType::ObjectRoutine => e.token(TokenKind::ROUTINE_KW),
        _ => e.token(TokenKind::FUNCTION_KW),
    }
    e.space();

    // Function name with arguments
    if let Some(ref func) = n.func {
        emit_object_with_args(e, func);
    }

    let dollar_hint = if n.objtype() == ObjectType::ObjectProcedure {
        super::DollarQuoteHint::Procedure
    } else {
        super::DollarQuoteHint::Function
    };

    // Actions (IMMUTABLE, SECURITY DEFINER, SET ..., etc.) are separated by whitespace, not commas.
    // Sort according to Postgres's canonical order
    if !n.actions.is_empty() {
        e.indent_start();
        for action in super::create_function_stmt::sort_function_options(&n.actions) {
            let def_elem = assert_node_variant!(DefElem, &action);
            e.line(LineType::SoftOrSpace);
            super::create_function_stmt::format_function_option(e, def_elem, dollar_hint);
        }
        e.indent_end();
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
