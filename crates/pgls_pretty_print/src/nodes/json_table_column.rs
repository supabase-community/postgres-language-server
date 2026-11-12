use pgls_query::protobuf::{
    JsonQuotes, JsonTableColumn, JsonTableColumnType, JsonWrapper, TypeName,
};

use super::prelude::*;

pub(super) fn emit_json_table_column(e: &mut EventEmitter, col: &JsonTableColumn) {
    e.group_start();

    if col.coltype() == JsonTableColumnType::JtcNested {
        e.token(TokenKind::NESTED_KW);
        e.space();
        e.token(TokenKind::PATH_KW);
        e.space();
        if let Some(pathspec) = col.pathspec.as_ref() {
            super::emit_json_table_path_spec(e, pathspec);
        }

        if !col.columns.is_empty() {
            e.line(LineType::SoftOrSpace);
            e.token(TokenKind::COLUMNS_KW);
            e.space();
            e.token(TokenKind::L_PAREN);
            e.indent_start();
            e.line(LineType::Soft);
            emit_comma_separated_list(e, &col.columns, |e, node| {
                if let Some(NodeEnum::JsonTableColumn(nested)) = node.node.as_ref() {
                    super::emit_json_table_column(e, nested);
                } else {
                    emit_node(e, node);
                }
            });
            e.indent_end();
            e.line(LineType::Soft);
            e.token(TokenKind::R_PAREN);
        }

        e.group_end();
        return;
    }

    if !col.name.is_empty() {
        emit_identifier_maybe_quoted(e, &col.name);
    }

    if col.coltype() == JsonTableColumnType::JtcForOrdinality {
        e.space();
        e.token(TokenKind::FOR_KW);
        e.space();
        e.token(TokenKind::ORDINALITY_KW);
        e.group_end();
        return;
    }

    if let Some(type_name) = col.type_name.as_ref() {
        e.space();
        if !emit_inline_type_name(e, type_name) {
            super::emit_type_name(e, type_name);
        }
    }

    if col.coltype() == JsonTableColumnType::JtcExists {
        e.space();
        e.token(TokenKind::EXISTS_KW);
    }

    if let Some(format) = col.format.as_ref() {
        emit_json_format_clause(e, format);
    }

    // The clauses of a column that doesn't fit continue on their own lines, one level in.
    e.indent_start();
    if let Some(pathspec) = col.pathspec.as_ref() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::PATH_KW);
        e.space();
        super::emit_json_table_path_spec(e, pathspec);
    }

    match col.wrapper() {
        JsonWrapper::JswNone => {
            if matches!(
                col.coltype(),
                JsonTableColumnType::JtcRegular
                    | JsonTableColumnType::JtcFormatted
                    | JsonTableColumnType::Undefined
            ) {
                e.line(LineType::SoftOrSpace);
                e.token(TokenKind::WITHOUT_KW);
                e.space();
                e.token(TokenKind::WRAPPER_KW);
            }
        }
        JsonWrapper::JswConditional => {
            e.line(LineType::SoftOrSpace);
            e.token(TokenKind::WITH_KW);
            e.space();
            e.token(TokenKind::CONDITIONAL_KW);
            e.space();
            e.token(TokenKind::WRAPPER_KW);
        }
        JsonWrapper::JswUnconditional => {
            e.line(LineType::SoftOrSpace);
            e.token(TokenKind::WITH_KW);
            e.space();
            e.token(TokenKind::UNCONDITIONAL_KW);
            e.space();
            e.token(TokenKind::WRAPPER_KW);
        }
        JsonWrapper::JswUnspec | JsonWrapper::Undefined => {}
    }

    match col.quotes() {
        JsonQuotes::JsQuotesKeep => {
            e.line(LineType::SoftOrSpace);
            e.token(TokenKind::KEEP_KW);
            e.space();
            e.token(TokenKind::QUOTES_KW);
        }
        JsonQuotes::JsQuotesOmit => {
            e.line(LineType::SoftOrSpace);
            e.token(TokenKind::OMIT_KW);
            e.space();
            e.token(TokenKind::QUOTES_KW);
        }
        JsonQuotes::JsQuotesUnspec | JsonQuotes::Undefined => {}
    }

    if let Some(on_empty) = col.on_empty.as_ref() {
        e.line(LineType::SoftOrSpace);
        emit_json_behavior_clause(e, on_empty, JsonBehaviorClause::OnEmpty);
    }

    if let Some(on_error) = col.on_error.as_ref() {
        e.line(LineType::SoftOrSpace);
        emit_json_behavior_clause(e, on_error, JsonBehaviorClause::OnError);
    }

    e.indent_end();
    e.group_end();
}

fn emit_inline_type_name(e: &mut EventEmitter, type_name: &TypeName) -> bool {
    if type_name.setof
        || type_name.pct_type
        || !type_name.typmods.is_empty()
        || !type_name.array_bounds.is_empty()
    {
        return false;
    }

    let mut parts = Vec::new();
    for node in &type_name.names {
        if let Some(NodeEnum::String(s)) = node.node.as_ref() {
            parts.push(s.sval.as_str());
        } else {
            return false;
        }
    }

    if parts.len() != 1 {
        return false;
    }

    emit_identifier_maybe_quoted(e, parts[0]);
    true
}
