use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind};
use pgt_query::protobuf::AlterFunctionStmt;

use super::node_list::emit_comma_separated_list;
use super::object_with_args::emit_object_with_args;

pub(super) fn emit_alter_function_stmt(e: &mut EventEmitter, n: &AlterFunctionStmt) {
    e.group_start(GroupKind::AlterFunctionStmt);

    e.token(TokenKind::ALTER_KW);
    e.space();

    // objtype: 0=FUNCTION, 1=PROCEDURE
    match n.objtype {
        1 => {
            e.token(TokenKind::IDENT("PROCEDURE".to_string()));
        }
        _ => {
            e.token(TokenKind::IDENT("FUNCTION".to_string()));
        }
    }
    e.space();

    // Function name with arguments
    if let Some(ref func) = n.func {
        emit_object_with_args(e, func);
    }

    // Emit actions (function options like IMMUTABLE, SECURITY DEFINER, etc.)
    if !n.actions.is_empty() {
        e.space();
        emit_comma_separated_list(e, &n.actions, |node, e| {
            let def_elem = assert_node_variant!(DefElem, node);
            super::create_function_stmt::format_function_option(e, def_elem);
        });
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
