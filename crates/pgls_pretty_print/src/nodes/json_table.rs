use pgls_query::protobuf::JsonTable;

use super::prelude::*;

pub(super) fn emit_json_table(e: &mut EventEmitter, n: &JsonTable) {
    e.group_start();

    if n.lateral {
        e.token(TokenKind::LATERAL_KW);
        e.space();
    }

    e.token(TokenKind::JSON_TABLE_KW);
    e.token(TokenKind::L_PAREN);
    e.indent_start();

    if let Some(context) = n.context_item.as_ref() {
        e.line(LineType::Soft);
        super::emit_json_value_expr(e, context);
        e.token(TokenKind::COMMA);
    }

    if let Some(pathspec) = n.pathspec.as_ref() {
        e.line(LineType::SoftOrSpace);
        super::emit_json_table_path_spec(e, pathspec);
    }

    if !n.passing.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::PASSING_KW);
        e.space();
        emit_comma_separated_list(e, &n.passing, |e, node| {
            let argument = assert_node_variant!(JsonArgument, node);
            super::emit_json_argument(e, argument);
        });
    }

    if !n.columns.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::COLUMNS_KW);
        e.space();
        e.token(TokenKind::L_PAREN);
        e.indent_start();
        e.line(LineType::Soft);
        emit_comma_separated_list(e, &n.columns, |e, node| {
            if let Some(NodeEnum::JsonTableColumn(col)) = node.node.as_ref() {
                super::emit_json_table_column(e, col);
            } else {
                emit_node(e, node);
            }
        });
        e.indent_end();
        e.line(LineType::Soft);
        e.token(TokenKind::R_PAREN);
    }

    if let Some(on_error) = n.on_error.as_ref() {
        e.line(LineType::SoftOrSpace);
        emit_json_behavior_clause(e, on_error, JsonBehaviorClause::OnError);
    }

    e.indent_end();
    e.line(LineType::Soft);
    e.token(TokenKind::R_PAREN);

    if let Some(alias) = n.alias.as_ref() {
        e.space();
        super::emit_alias(e, alias);
    }

    e.group_end();
}
