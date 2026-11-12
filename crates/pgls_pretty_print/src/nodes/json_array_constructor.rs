use pgls_query::protobuf::JsonArrayConstructor;

use super::emit_json_value_expr;
use super::prelude::*;

pub(super) fn emit_json_array_constructor(e: &mut EventEmitter, n: &JsonArrayConstructor) {
    e.group_start();

    e.token(TokenKind::JSON_ARRAY_KW);
    e.token(TokenKind::L_PAREN);

    if !n.exprs.is_empty() {
        emit_comma_separated_list(e, &n.exprs, |emitter, node| {
            if let Some(pgls_query::NodeEnum::JsonValueExpr(value)) = node.node.as_ref() {
                emit_json_value_expr(emitter, value);
            } else {
                emit_node(emitter, node);
            }
        });
    }

    // The default for JSON_ARRAY without a null clause is ABSENT ON NULL
    // So we only need to emit NULL ON NULL when absent_on_null is false
    if !n.exprs.is_empty() {
        e.line(crate::emitter::LineType::SoftOrSpace);
        if n.absent_on_null {
            e.token(TokenKind::ABSENT_KW);
        } else {
            e.token(TokenKind::NULL_KW);
        }
        e.space();
        e.token(TokenKind::ON_KW);
        e.space();
        e.token(TokenKind::NULL_KW);
    }

    if let Some(ref output) = n.output {
        let mut guard = !n.exprs.is_empty();
        emit_json_output_clause(e, output, &mut guard);
    }

    e.token(TokenKind::R_PAREN);

    e.group_end();
}
