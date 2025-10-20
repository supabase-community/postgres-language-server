use pgt_query::protobuf::{CoercionForm, RowExpr};

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

use super::node_list::emit_comma_separated_list;

pub(super) fn emit_row_expr(e: &mut EventEmitter, n: &RowExpr) {
    e.group_start(GroupKind::RowExpr);

    let format = n.row_format();
    let emit_row_keyword = matches!(
        format,
        CoercionForm::CoerceExplicitCall | CoercionForm::CoerceSqlSyntax
    );

    if emit_row_keyword {
        e.token(TokenKind::ROW_KW);
    }

    e.token(TokenKind::L_PAREN);
    emit_comma_separated_list(e, &n.args, super::emit_node);
    e.token(TokenKind::R_PAREN);

    if !n.colnames.is_empty() {
        e.space();
        e.token(TokenKind::AS_KW);
        e.space();
        e.token(TokenKind::L_PAREN);
        emit_comma_separated_list(e, &n.colnames, super::emit_node);
        e.token(TokenKind::R_PAREN);
    }

    e.group_end();
}
