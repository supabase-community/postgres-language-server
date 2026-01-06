use pgls_query::protobuf::{ViewCheckOption, ViewStmt};

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
};

use super::node_list::emit_comma_separated_list;

pub(super) fn emit_view_stmt(e: &mut EventEmitter, n: &ViewStmt) {
    e.group_start(GroupKind::ViewStmt);

    e.token(TokenKind::CREATE_KW);

    if n.replace {
        e.space();
        e.token(TokenKind::OR_KW);
        e.space();
        e.token(TokenKind::REPLACE_KW);
    }

    if let Some(ref view) = n.view {
        match view.relpersistence.as_str() {
            "t" => {
                e.space();
                e.token(TokenKind::TEMPORARY_KW);
            }
            "u" => {
                e.space();
                e.token(TokenKind::UNLOGGED_KW);
            }
            _ => {}
        }
    }

    e.space();
    e.token(TokenKind::VIEW_KW);

    if let Some(ref view) = n.view {
        e.space();
        super::emit_range_var(e, view);
    }

    // Column aliases
    if !n.aliases.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::L_PAREN);
        e.line(LineType::Soft);
        e.indent_start();
        emit_comma_separated_list(e, &n.aliases, |alias_node, emitter| {
            let alias = assert_node_variant!(String, alias_node);
            super::string::emit_identifier_maybe_quoted(emitter, &alias.sval);
        });
        e.indent_end();
        e.line(LineType::Soft);
        e.token(TokenKind::R_PAREN);
    }

    if !n.options.is_empty() {
        e.line(LineType::Hard);
        e.indent_start();
        e.token(TokenKind::WITH_KW);
        e.space();
        e.token(TokenKind::L_PAREN);
        emit_comma_separated_list(e, &n.options, super::emit_node);
        e.token(TokenKind::R_PAREN);
        e.indent_end();
    }

    // Query
    if let Some(ref query) = n.query {
        e.line(LineType::Hard);
        e.token(TokenKind::AS_KW);
        e.space();

        if let Some(pgls_query::NodeEnum::SelectStmt(stmt)) = query.node.as_ref() {
            super::emit_select_stmt_no_semicolon(e, stmt);
        } else {
            super::emit_node(query, e);
        }
    }

    // WITH CHECK OPTION
    match n.with_check_option() {
        ViewCheckOption::LocalCheckOption => {
            e.line(LineType::SoftOrSpace);
            e.token(TokenKind::WITH_KW);
            e.space();
            e.token(TokenKind::LOCAL_KW);
            e.space();
            e.token(TokenKind::CHECK_KW);
            e.space();
            e.token(TokenKind::OPTION_KW);
        }
        ViewCheckOption::CascadedCheckOption => {
            e.line(LineType::SoftOrSpace);
            e.token(TokenKind::WITH_KW);
            e.space();
            e.token(TokenKind::CASCADED_KW);
            e.space();
            e.token(TokenKind::CHECK_KW);
            e.space();
            e.token(TokenKind::OPTION_KW);
        }
        ViewCheckOption::NoCheckOption => {
            // No check option
        }
        ViewCheckOption::Undefined => {
            // Undefined, don't emit
        }
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
