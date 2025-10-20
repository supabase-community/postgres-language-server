use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind};
use pgt_query::protobuf::AlterEnumStmt;

use super::node_list::emit_dot_separated_list;
use super::string::{emit_keyword, emit_single_quoted_str};

pub(super) fn emit_alter_enum_stmt(e: &mut EventEmitter, n: &AlterEnumStmt) {
    e.group_start(GroupKind::AlterEnumStmt);

    e.token(TokenKind::ALTER_KW);
    e.space();
    emit_keyword(e, "TYPE");
    e.space();

    // Enum type name (qualified)
    if !n.type_name.is_empty() {
        emit_dot_separated_list(e, &n.type_name);
    }

    e.space();

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
            e.token(TokenKind::IF_KW);
            e.space();
            e.token(TokenKind::NOT_KW);
            e.space();
            e.token(TokenKind::EXISTS_KW);
        }

        if !n.new_val.is_empty() {
            e.space();
            emit_single_quoted_str(e, &n.new_val);
        }

        // Optional BEFORE/AFTER clause
        if !n.new_val_neighbor.is_empty() {
            e.space();
            if n.new_val_is_after {
                emit_keyword(e, "AFTER");
            } else {
                emit_keyword(e, "BEFORE");
            }
            e.space();
            emit_single_quoted_str(e, &n.new_val_neighbor);
        }
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
