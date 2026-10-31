use pgls_query::protobuf::JsonSerializeExpr;

use super::emit_json_value_expr;
use super::json_value_expr::emit_json_output;
use super::prelude::*;

pub(super) fn emit_json_serialize_expr(e: &mut EventEmitter, n: &JsonSerializeExpr) {
    e.group_start();

    e.token(TokenKind::JSON_SERIALIZE_KW);
    e.token(TokenKind::L_PAREN);

    let mut has_content = false;

    if let Some(ref expr) = n.expr {
        emit_json_value_expr(e, expr);
        has_content = true;
    }

    if let Some(ref output) = n.output {
        emit_json_output(e, output, &mut has_content);
    }

    e.token(TokenKind::R_PAREN);

    e.group_end();
}
