use pgls_query::protobuf::JsonParseExpr;

use super::emit_json_value_expr;
use super::json_value_expr::emit_json_output;
use super::prelude::*;

pub(super) fn emit_json_parse_expr(e: &mut EventEmitter, n: &JsonParseExpr) {
    e.group_start();

    e.token(TokenKind::JSON_KW);
    e.token(TokenKind::L_PAREN);

    let mut has_content = false;

    if let Some(ref expr) = n.expr {
        emit_json_value_expr(e, expr);
        has_content = true;
    }

    if n.unique_keys {
        if has_content {
            e.space();
        }
        e.token(TokenKind::WITH_KW);
        e.space();
        e.token(TokenKind::UNIQUE_KW);
        e.space();
        e.token(TokenKind::KEYS_KW);
        has_content = true;
    }

    if let Some(ref output) = n.output {
        emit_json_output(e, output, &mut has_content);
    }

    e.token(TokenKind::R_PAREN);

    e.group_end();
}
