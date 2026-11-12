use pgls_query::protobuf::CreateConversionStmt;

use super::prelude::*;

pub(super) fn emit_create_conversion_stmt(e: &mut EventEmitter, n: &CreateConversionStmt) {
    e.group_start();

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

    e.line(LineType::SoftOrSpace);
    e.token(TokenKind::FOR_KW);
    e.space();
    emit_single_quoted_str(e, &n.for_encoding_name);
    e.space();
    e.token(TokenKind::TO_KW);
    e.space();
    emit_single_quoted_str(e, &n.to_encoding_name);
    e.line(LineType::SoftOrSpace);
    e.token(TokenKind::FROM_KW);
    e.space();

    // Function name
    emit_dot_separated_list(e, &n.func_name);

    e.group_end();
}
