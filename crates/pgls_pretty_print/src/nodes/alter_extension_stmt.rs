use pgls_query::protobuf::AlterExtensionStmt;

use super::prelude::*;

pub(super) fn emit_alter_extension_stmt(e: &mut EventEmitter, n: &AlterExtensionStmt) {
    e.group_start();

    e.token(TokenKind::ALTER_KW);
    e.space();
    e.token(TokenKind::EXTENSION_KW);
    e.space();

    if !n.extname.is_empty() {
        emit_identifier_maybe_quoted(e, &n.extname);
    }

    // `UPDATE` is written with or without a version: `UPDATE [TO version]`.
    // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/gram.y#L5280
    e.line(LineType::SoftOrSpace);
    e.token(TokenKind::UPDATE_KW);
    for opt in &n.options {
        e.space();
        if let Some(pgls_query::NodeEnum::DefElem(d)) = &opt.node
            && d.defname == "new_version"
        {
            e.token(TokenKind::TO_KW);
            if let Some(ref arg) = d.arg {
                e.space();
                // Version must be a string literal (quoted)
                if let Some(pgls_query::NodeEnum::String(s)) = &arg.node {
                    emit_string_literal(e, s);
                } else {
                    emit_node(e, arg);
                }
            }
        } else {
            emit_node(e, opt);
        }
    }

    e.group_end();
}
