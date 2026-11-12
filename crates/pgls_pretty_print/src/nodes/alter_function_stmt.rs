use pgls_query::protobuf::AlterFunctionStmt;

use super::emit_object_with_args;
use super::prelude::*;

pub(super) fn emit_alter_function_stmt(e: &mut EventEmitter, n: &AlterFunctionStmt) {
    e.group_start();

    e.token(TokenKind::ALTER_KW);
    e.space();

    emit_keywords(e, object_type_keywords(n.objtype()).unwrap_or("FUNCTION"));
    e.space();

    // Function name with arguments
    if let Some(ref func) = n.func {
        emit_object_with_args(e, func);
    }

    // Actions (IMMUTABLE, SECURITY DEFINER, SET ..., etc.) are separated by whitespace, not commas.
    // Sort according to Postgres's canonical order
    if !n.actions.is_empty() {
        e.indent_start();
        for action in sort_function_options(&n.actions) {
            let Some(NodeEnum::DefElem(def_elem)) = action.node.as_ref() else {
                emit_node(e, &action);
                continue;
            };
            e.line(LineType::SoftOrSpace);
            format_function_option(e, def_elem);
        }
        e.indent_end();
    }

    e.group_end();
}
