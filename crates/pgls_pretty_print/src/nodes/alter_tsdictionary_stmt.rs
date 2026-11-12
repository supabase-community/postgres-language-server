use pgls_query::protobuf::AlterTsDictionaryStmt;

use super::prelude::*;

pub(super) fn emit_alter_tsdictionary_stmt(e: &mut EventEmitter, n: &AlterTsDictionaryStmt) {
    e.group_start(GroupKind::AlterTsdictionaryStmt);

    e.token(TokenKind::ALTER_KW);
    e.space();
    e.token(TokenKind::TEXT_KW);
    e.space();
    e.token(TokenKind::SEARCH_KW);
    e.space();
    e.token(TokenKind::DICTIONARY_KW);
    e.space();

    // Dictionary name
    emit_dot_separated_list(e, &n.dictname);

    // Options
    if !n.options.is_empty() {
        e.space();
        e.token(TokenKind::L_PAREN);
        e.line(LineType::Soft);
        e.indent_start();
        emit_comma_separated_list(e, &n.options, emit_node);
        e.indent_end();
        e.line(LineType::Soft);
        e.token(TokenKind::R_PAREN);
    }

    e.group_end();
}
