use pgls_query::protobuf::MergeSupportFunc;

use super::prelude::*;

/// `merge_action()`, the only merge support function. The parser makes it a `MergeSupportFunc`
/// rather than a function call; its type and collation are fixed, so there is nothing else to
/// print.
/// https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/gram.y#L16208
pub(super) fn emit_merge_support_func(e: &mut EventEmitter, _n: &MergeSupportFunc) {
    e.group_start();

    e.token(TokenKind::MERGE_ACTION_KW);
    e.token(TokenKind::L_PAREN);
    e.token(TokenKind::R_PAREN);

    e.group_end();
}
