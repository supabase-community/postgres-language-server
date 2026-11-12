use pgls_query::protobuf::JsonScalarExpr;

use super::json_value_expr::emit_json_output;
use super::prelude::*;

pub(super) fn emit_json_scalar_expr(e: &mut EventEmitter, n: &JsonScalarExpr) {
    e.group_start(GroupKind::JsonScalarExpr);

    e.token(TokenKind::JSON_SCALAR_KW);
    e.token(TokenKind::L_PAREN);

    let mut has_content = false;

    if let Some(ref expr) = n.expr {
        emit_node(expr, e);
        has_content = true;
    }

    if let Some(ref output) = n.output {
        emit_json_output(e, output, &mut has_content);
    }

    e.token(TokenKind::R_PAREN);

    e.group_end();
}
