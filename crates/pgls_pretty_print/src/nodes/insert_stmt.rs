use pgls_query::protobuf::{InsertStmt, OverridingKind};

use super::prelude::*;

pub(super) fn emit_insert_stmt(e: &mut EventEmitter, n: &InsertStmt) {
    e.group_start();

    if let Some(ref with_clause) = n.with_clause {
        super::emit_with_clause(e, with_clause);
        e.line(LineType::SoftOrSpace);
    }

    e.token(TokenKind::INSERT_KW);
    e.space();
    e.token(TokenKind::INTO_KW);
    e.space();

    // Emit table name
    if let Some(ref relation) = n.relation {
        super::emit_range_var(e, relation);
    }

    // Emit column list if present
    if !n.cols.is_empty() {
        e.space();
        // The column list has its own group so it can fit on one line in fit layout. In expanded
        // layout the separator breaks below are hard, which makes the group open up on purpose.
        e.group_start();
        e.token(TokenKind::L_PAREN);
        e.line(LineType::Soft);
        e.indent_start();

        emit_comma_separated_list_with_layout_break(e, &n.cols, |e, node| {
            if let Some(pgls_query::NodeEnum::ResTarget(res_target)) = node.node.as_ref() {
                emit_column_name(e, res_target);
            } else {
                emit_node(e, node);
            }
        });

        e.indent_end();
        e.line(LineType::Soft);
        e.token(TokenKind::R_PAREN);
        e.group_end();
    }

    match n.r#override() {
        OverridingKind::OverridingUserValue => {
            e.line(LineType::SoftOrSpace);
            e.token(TokenKind::OVERRIDING_KW);
            e.space();
            e.token(TokenKind::USER_KW);
            e.space();
            e.token(TokenKind::VALUE_KW);
        }
        OverridingKind::OverridingSystemValue => {
            e.line(LineType::SoftOrSpace);
            e.token(TokenKind::OVERRIDING_KW);
            e.space();
            e.token(TokenKind::SYSTEM_KW);
            e.space();
            e.token(TokenKind::VALUE_KW);
        }
        OverridingKind::OverridingNotSet | OverridingKind::Undefined => {}
    }

    // Emit VALUES or SELECT or DEFAULT VALUES
    if let Some(ref select_stmt) = n.select_stmt {
        emit_layout_break(e);
        emit_node(e, select_stmt);
    } else {
        // No select_stmt means DEFAULT VALUES
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::DEFAULT_KW);
        e.space();
        e.token(TokenKind::VALUES_KW);
    }

    // Emit ON CONFLICT clause if present
    if let Some(ref on_conflict) = n.on_conflict_clause {
        super::emit_on_conflict_clause(e, on_conflict);
    }

    if !n.returning_list.is_empty() {
        emit_layout_break(e);
        e.token(TokenKind::RETURNING_KW);
        e.space();
        emit_comma_separated_list(e, &n.returning_list, emit_node);
    }

    e.group_end();
}
