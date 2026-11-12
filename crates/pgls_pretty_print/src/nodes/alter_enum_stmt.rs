use pgls_query::protobuf::AlterEnumStmt;

use super::prelude::*;

pub(super) fn emit_alter_enum_stmt(e: &mut EventEmitter, n: &AlterEnumStmt) {
    e.group_start();

    e.token(TokenKind::ALTER_KW);
    e.space();
    emit_keyword(e, "TYPE");
    e.space();

    // Enum type name (qualified)
    if !n.type_name.is_empty() {
        emit_dot_separated_list(e, &n.type_name);
    }

    e.line(LineType::SoftOrSpace);

    // Check if this is ADD VALUE or RENAME VALUE
    if !n.old_val.is_empty() {
        // RENAME VALUE old TO new
        emit_keyword(e, "RENAME");
        e.space();
        emit_keyword(e, "VALUE");
        e.space();
        emit_single_quoted_str(e, &n.old_val);
        e.space();
        e.token(TokenKind::TO_KW);
        e.space();
        emit_single_quoted_str(e, &n.new_val);
    } else {
        // ADD VALUE [ IF NOT EXISTS ] new_value [ BEFORE old_value | AFTER old_value ]
        e.token(TokenKind::ADD_KW);
        e.space();
        emit_keyword(e, "VALUE");

        if n.skip_if_new_val_exists {
            e.space();
            emit_if_not_exists(e);
        }

        if !n.new_val.is_empty() {
            e.space();
            emit_single_quoted_str(e, &n.new_val);
        }

        // Optional BEFORE/AFTER clause
        if !n.new_val_neighbor.is_empty() {
            e.line(LineType::SoftOrSpace);
            if n.new_val_is_after {
                emit_keyword(e, "AFTER");
            } else {
                emit_keyword(e, "BEFORE");
            }
            e.space();
            emit_single_quoted_str(e, &n.new_val_neighbor);
        }
    }

    e.group_end();
}
