use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind, LineType};
use pgls_query::protobuf::AlterFunctionStmt;

use super::node_list::emit_comma_separated_list;
use super::object_with_args::emit_object_with_args;

pub(super) fn emit_alter_function_stmt(e: &mut EventEmitter, n: &AlterFunctionStmt) {
    e.group_start(GroupKind::AlterFunctionStmt);

    e.token(TokenKind::ALTER_KW);
    e.space();

    // ObjectType: ObjectFunction=20, ObjectProcedure=30
    match n.objtype {
        30 => {
            e.token(TokenKind::PROCEDURE_KW);
        }
        _ => {
            e.token(TokenKind::FUNCTION_KW);
        }
    }
    e.line(LineType::SoftOrSpace);

    // Function name with arguments
    if let Some(ref func) = n.func {
        emit_object_with_args(e, func);
    }

    // Determine the dollar quote hint based on whether this is a procedure or function
    // ObjectType: ObjectFunction=20, ObjectProcedure=30
    let dollar_hint = if n.objtype == 30 {
        super::DollarQuoteHint::Procedure
    } else {
        super::DollarQuoteHint::Function
    };

    // Emit actions (function options like IMMUTABLE, SECURITY DEFINER, etc.)
    // Sort according to Postgres's canonical order
    if !n.actions.is_empty() {
        e.line(LineType::SoftOrSpace);
        let sorted_actions = super::create_function_stmt::sort_function_options(&n.actions);
        emit_comma_separated_list(e, &sorted_actions, |node, e| {
            let def_elem = assert_node_variant!(DefElem, node);
            super::create_function_stmt::format_function_option(e, def_elem, dollar_hint);
        });
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
