use pgls_query::protobuf::JsonArrayQueryConstructor;

use super::prelude::*;

pub(super) fn emit_json_array_query_constructor(
    e: &mut EventEmitter,
    n: &JsonArrayQueryConstructor,
) {
    e.group_start();

    e.token(TokenKind::JSON_ARRAY_KW);
    e.token(TokenKind::L_PAREN);

    let mut has_content = false;

    if let Some(ref query) = n.query {
        emit_node(e, query);
        has_content = true;
    }

    // Note: For JSON_ARRAY with subquery, ABSENT ON NULL is always the default
    // and cannot be overridden, so we don't emit it

    if let Some(ref format) = n.format {
        emit_json_format_clause(e, format);
        has_content = true;
    }

    if let Some(ref output) = n.output {
        emit_json_output_clause(e, output, &mut has_content);
    }

    e.token(TokenKind::R_PAREN);

    e.group_end();
}
