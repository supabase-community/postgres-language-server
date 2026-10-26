use pgls_query::protobuf::JsonObjectConstructor;

use super::prelude::*;

pub(super) fn emit_json_object_constructor(e: &mut EventEmitter, n: &JsonObjectConstructor) {
    e.group_start();

    e.token(TokenKind::JSON_OBJECT_KW);
    e.token(TokenKind::L_PAREN);

    let mut has_content = false;

    if !n.exprs.is_empty() {
        emit_comma_separated_list(e, &n.exprs, emit_node);
        has_content = true;
    }

    if n.absent_on_null {
        if has_content {
            e.space();
        }
        e.token(TokenKind::ABSENT_KW);
        e.space();
        e.token(TokenKind::ON_KW);
        e.space();
        e.token(TokenKind::NULL_KW);
        has_content = true;
    }

    if n.unique {
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
        emit_json_output_clause(e, output, &mut has_content);
    }

    e.token(TokenKind::R_PAREN);

    e.group_end();
}
