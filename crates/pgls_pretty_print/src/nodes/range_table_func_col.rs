use pgls_query::protobuf::RangeTableFuncCol;

use super::prelude::*;

pub(super) fn emit_range_table_func_col(e: &mut EventEmitter, col: &RangeTableFuncCol) {
    e.group_start();

    if !col.colname.is_empty() {
        emit_identifier_maybe_quoted(e, &col.colname);
    }

    if col.for_ordinality {
        if !col.colname.is_empty() {
            e.space();
        }
        e.token(TokenKind::FOR_KW);
        e.space();
        e.token(TokenKind::ORDINALITY_KW);
        e.group_end();
        return;
    }

    if let Some(type_name) = col.type_name.as_ref() {
        if !col.colname.is_empty() {
            e.space();
        }
        super::emit_type_name(e, type_name);
    }

    if let Some(expr) = col.colexpr.as_ref() {
        e.space();
        e.token(TokenKind::PATH_KW);
        e.space();
        emit_node(e, expr);
    }

    if let Some(expr) = col.coldefexpr.as_ref() {
        e.space();
        e.token(TokenKind::DEFAULT_KW);
        e.space();
        emit_node(e, expr);
    }

    if col.is_not_null {
        e.space();
        e.token(TokenKind::NOT_KW);
        e.space();
        e.token(TokenKind::NULL_KW);
    }

    e.group_end();
}
