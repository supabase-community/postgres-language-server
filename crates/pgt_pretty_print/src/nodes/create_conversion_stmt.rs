use super::{
    node_list::emit_dot_separated_list,
    string::{emit_keyword, emit_single_quoted_str},
};
use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};
use pgt_query::protobuf::CreateConversionStmt;

pub(super) fn emit_create_conversion_stmt(e: &mut EventEmitter, n: &CreateConversionStmt) {
    e.group_start(GroupKind::CreateConversionStmt);

    e.token(TokenKind::CREATE_KW);
    e.space();

    if n.def {
        e.token(TokenKind::DEFAULT_KW);
        e.space();
    }

    emit_keyword(e, "CONVERSION");
    e.space();

    // Conversion name
    emit_dot_separated_list(e, &n.conversion_name);

    e.space();
    e.token(TokenKind::FOR_KW);
    e.space();
    emit_single_quoted_str(e, &n.for_encoding_name);
    e.space();
    e.token(TokenKind::TO_KW);
    e.space();
    emit_single_quoted_str(e, &n.to_encoding_name);
    e.space();
    e.token(TokenKind::FROM_KW);
    e.space();

    // Function name
    emit_dot_separated_list(e, &n.func_name);

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
