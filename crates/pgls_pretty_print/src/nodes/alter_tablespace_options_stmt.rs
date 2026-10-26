use pgls_query::protobuf::AlterTableSpaceOptionsStmt;

use super::prelude::*;

pub(super) fn emit_alter_tablespace_options_stmt(
    e: &mut EventEmitter,
    n: &AlterTableSpaceOptionsStmt,
) {
    e.group_start(GroupKind::AlterTableSpaceOptionsStmt);

    e.token(TokenKind::ALTER_KW);
    e.space();
    e.token(TokenKind::TABLESPACE_KW);
    e.space();
    emit_identifier_maybe_quoted(e, &n.tablespacename);

    e.line(LineType::SoftOrSpace);
    if n.is_reset {
        e.token(TokenKind::RESET_KW);
    } else {
        e.token(TokenKind::SET_KW);
    }

    if !n.options.is_empty() {
        e.space();
        emit_parenthesized_list(e, &n.options, emit_node);
    }

    e.group_end();
}
