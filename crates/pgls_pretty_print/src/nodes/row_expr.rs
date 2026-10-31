use pgls_query::protobuf::{CoercionForm, RowExpr};

use super::prelude::*;

pub(super) fn emit_row_expr(e: &mut EventEmitter, n: &RowExpr) {
    e.group_start();

    let format = n.row_format();
    let emit_row_keyword = matches!(
        format,
        CoercionForm::CoerceExplicitCall | CoercionForm::CoerceSqlSyntax
    );

    if emit_row_keyword {
        e.token(TokenKind::ROW_KW);
    }

    emit_parenthesized_list(e, &n.args, emit_node);

    if !n.colnames.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::AS_KW);
        e.space();
        emit_parenthesized_list(e, &n.colnames, emit_node);
    }

    e.group_end();
}
