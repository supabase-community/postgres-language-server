use pgls_query::protobuf::JsonValueExpr;

use super::prelude::*;

pub(super) fn emit_json_value_expr(e: &mut EventEmitter, n: &JsonValueExpr) {
    e.group_start();

    if let Some(ref raw_expr) = n.raw_expr {
        emit_node(e, raw_expr);
    } else if let Some(ref formatted_expr) = n.formatted_expr {
        emit_node(e, formatted_expr);
    }

    if let Some(ref format) = n.format {
        emit_json_format_clause(e, format);
    }

    e.group_end();
}
