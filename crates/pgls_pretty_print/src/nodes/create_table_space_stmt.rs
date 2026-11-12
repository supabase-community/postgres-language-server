use pgls_query::protobuf::CreateTableSpaceStmt;

use super::prelude::*;

pub(super) fn emit_create_table_space_stmt(e: &mut EventEmitter, n: &CreateTableSpaceStmt) {
    e.group_start();

    e.token(TokenKind::CREATE_KW);
    e.space();
    e.token(TokenKind::TABLESPACE_KW);

    if !n.tablespacename.is_empty() {
        e.space();
        emit_identifier_maybe_quoted(e, &n.tablespacename);
    }

    // OWNER
    if let Some(ref owner) = n.owner {
        e.space();
        emit_keyword(e, "OWNER");
        e.space();
        super::emit_role_spec(e, owner);
    }

    // LOCATION (always required in CREATE TABLESPACE, even if empty string)
    e.space();
    emit_keyword(e, "LOCATION");
    e.space();
    emit_single_quoted_str(e, &n.location);

    // WITH options
    if !n.options.is_empty() {
        e.space();
        emit_with_options(e, &n.options);
    }

    e.group_end();
}
