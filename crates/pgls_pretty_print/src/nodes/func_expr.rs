use pgls_query::protobuf::FuncExpr;

use super::prelude::*;

/// Emit a FuncExpr (planner function call node)
/// These are internal planner representations with function OIDs
/// We emit a simple fallback representation
pub(super) fn emit_func_expr(e: &mut EventEmitter, n: &FuncExpr) {
    e.group_start();

    // FuncExpr is the planner's representation of function calls
    // Without access to pg_proc, we emit a placeholder with the OID
    if n.funcid != 0 {
        e.token(TokenKind::IDENT(format!("func#{}", n.funcid)));
    } else {
        e.token(TokenKind::IDENT("func".to_string()));
    }

    e.token(TokenKind::L_PAREN);
    if !n.args.is_empty() {
        emit_comma_separated_list(e, &n.args, emit_node);
    }
    e.token(TokenKind::R_PAREN);

    if n.funcretset {
        e.space();
        e.token(TokenKind::SET_KW);
    }

    e.group_end();
}
