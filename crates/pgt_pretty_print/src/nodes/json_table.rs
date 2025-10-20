use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
    nodes::node_list::emit_comma_separated_list,
};
use pgt_query::{
    NodeEnum,
    protobuf::{
        JsonArgument, JsonBehavior, JsonBehaviorType, JsonQuotes, JsonTable, JsonTableColumn,
        JsonTableColumnType, JsonTablePathSpec, JsonWrapper, TypeName,
    },
};

pub(super) fn emit_json_table(e: &mut EventEmitter, n: &JsonTable) {
    e.group_start(GroupKind::JsonTable);

    if n.lateral {
        e.token(TokenKind::LATERAL_KW);
        e.space();
    }

    e.token(TokenKind::IDENT("JSON_TABLE".to_string()));
    e.token(TokenKind::L_PAREN);
    e.indent_start();

    if let Some(context) = n.context_item.as_ref() {
        e.line(LineType::SoftOrSpace);
        super::emit_json_value_expr(e, context);
        e.token(TokenKind::COMMA);
    }

    if let Some(pathspec) = n.pathspec.as_ref() {
        e.line(LineType::SoftOrSpace);
        emit_json_table_path_spec(e, pathspec);
    }

    if !n.passing.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::IDENT("PASSING".to_string()));
        e.space();
        emit_comma_separated_list(e, &n.passing, |node, e| {
            let argument = assert_node_variant!(JsonArgument, node);
            emit_json_argument(e, argument);
        });
    }

    if !n.columns.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::IDENT("COLUMNS".to_string()));
        e.space();
        e.token(TokenKind::L_PAREN);
        e.indent_start();
        e.line(LineType::SoftOrSpace);
        emit_comma_separated_list(e, &n.columns, |node, e| {
            if let Some(NodeEnum::JsonTableColumn(col)) = node.node.as_ref() {
                emit_json_table_column(e, col);
            } else {
                super::emit_node(node, e);
            }
        });
        e.indent_end();
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::R_PAREN);
    }

    if let Some(on_error) = n.on_error.as_ref() {
        e.line(LineType::SoftOrSpace);
        emit_json_behavior_clause(e, on_error, JsonBehaviorClause::OnError);
    }

    e.indent_end();
    e.line(LineType::SoftOrSpace);
    e.token(TokenKind::R_PAREN);

    if let Some(alias) = n.alias.as_ref() {
        e.space();
        super::emit_alias(e, alias);
    }

    e.group_end();
}

fn emit_json_table_path_spec(e: &mut EventEmitter, spec: &JsonTablePathSpec) {
    if let Some(string_node) = spec.string.as_ref() {
        super::emit_node(string_node, e);
    }

    if !spec.name.is_empty() {
        e.space();
        e.token(TokenKind::AS_KW);
        e.space();
        super::emit_identifier_maybe_quoted(e, &spec.name);
    }
}

fn emit_json_table_column(e: &mut EventEmitter, col: &JsonTableColumn) {
    e.group_start(GroupKind::JsonTableColumn);

    match col.coltype() {
        JsonTableColumnType::JtcNested => {
            e.token(TokenKind::IDENT("NESTED".to_string()));
            e.space();
            e.token(TokenKind::IDENT("PATH".to_string()));
            e.space();
            if let Some(pathspec) = col.pathspec.as_ref() {
                emit_json_table_path_spec(e, pathspec);
            }

            if !col.columns.is_empty() {
                e.line(LineType::SoftOrSpace);
                e.token(TokenKind::IDENT("COLUMNS".to_string()));
                e.space();
                e.token(TokenKind::L_PAREN);
                e.indent_start();
                e.line(LineType::SoftOrSpace);
                emit_comma_separated_list(e, &col.columns, |node, e| {
                    if let Some(NodeEnum::JsonTableColumn(nested)) = node.node.as_ref() {
                        emit_json_table_column(e, nested);
                    } else {
                        super::emit_node(node, e);
                    }
                });
                e.indent_end();
                e.line(LineType::SoftOrSpace);
                e.token(TokenKind::R_PAREN);
            }

            e.group_end();
            return;
        }
        _ => {}
    }

    if !col.name.is_empty() {
        super::emit_identifier_maybe_quoted(e, &col.name);
    }

    if col.coltype() == JsonTableColumnType::JtcForOrdinality {
        e.space();
        e.token(TokenKind::FOR_KW);
        e.space();
        e.token(TokenKind::IDENT("ORDINALITY".to_string()));
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
        e.token(TokenKind::IDENT("EXISTS".to_string()));
    }

    if let Some(format) = col.format.as_ref() {
        super::json_value_expr::emit_json_format(e, format);
    }

    if let Some(pathspec) = col.pathspec.as_ref() {
        e.space();
        e.token(TokenKind::IDENT("PATH".to_string()));
        e.space();
        emit_json_table_path_spec(e, pathspec);
    }

    match col.wrapper() {
        JsonWrapper::JswNone => {
            if matches!(
                col.coltype(),
                JsonTableColumnType::JtcRegular
                    | JsonTableColumnType::JtcFormatted
                    | JsonTableColumnType::Undefined
            ) {
                e.space();
                e.token(TokenKind::WITHOUT_KW);
                e.space();
                e.token(TokenKind::WRAPPER_KW);
            }
        }
        JsonWrapper::JswConditional => {
            e.space();
            e.token(TokenKind::WITH_KW);
            e.space();
            e.token(TokenKind::IDENT("CONDITIONAL".to_string()));
            e.space();
            e.token(TokenKind::WRAPPER_KW);
        }
        JsonWrapper::JswUnconditional => {
            e.space();
            e.token(TokenKind::WITH_KW);
            e.space();
            e.token(TokenKind::IDENT("UNCONDITIONAL".to_string()));
            e.space();
            e.token(TokenKind::WRAPPER_KW);
        }
        JsonWrapper::JswUnspec | JsonWrapper::Undefined => {}
    }

    match col.quotes() {
        JsonQuotes::JsQuotesKeep => {
            e.space();
            e.token(TokenKind::IDENT("KEEP".to_string()));
            e.space();
            e.token(TokenKind::IDENT("QUOTES".to_string()));
        }
        JsonQuotes::JsQuotesOmit => {
            e.space();
            e.token(TokenKind::IDENT("OMIT".to_string()));
            e.space();
            e.token(TokenKind::IDENT("QUOTES".to_string()));
        }
        JsonQuotes::JsQuotesUnspec | JsonQuotes::Undefined => {}
    }

    if let Some(on_empty) = col.on_empty.as_ref() {
        e.space();
        emit_json_behavior_clause(e, on_empty, JsonBehaviorClause::OnEmpty);
    }

    if let Some(on_error) = col.on_error.as_ref() {
        e.space();
        emit_json_behavior_clause(e, on_error, JsonBehaviorClause::OnError);
    }

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

    super::emit_identifier_maybe_quoted(e, parts[0]);
    true
}

fn emit_json_argument(e: &mut EventEmitter, argument: &JsonArgument) {
    if let Some(value) = argument.val.as_ref() {
        super::emit_json_value_expr(e, value);
    }

    if !argument.name.is_empty() {
        e.space();
        e.token(TokenKind::AS_KW);
        e.space();
        super::emit_identifier_maybe_quoted(e, &argument.name);
    }
}

fn emit_json_behavior(e: &mut EventEmitter, behavior: &JsonBehavior) {
    match behavior.btype() {
        JsonBehaviorType::JsonBehaviorNull => e.token(TokenKind::NULL_KW),
        JsonBehaviorType::JsonBehaviorError => e.token(TokenKind::IDENT("ERROR".to_string())),
        JsonBehaviorType::JsonBehaviorEmpty => e.token(TokenKind::IDENT("EMPTY".to_string())),
        JsonBehaviorType::JsonBehaviorTrue => e.token(TokenKind::TRUE_KW),
        JsonBehaviorType::JsonBehaviorFalse => e.token(TokenKind::FALSE_KW),
        JsonBehaviorType::JsonBehaviorUnknown => e.token(TokenKind::UNKNOWN_KW),
        JsonBehaviorType::JsonBehaviorEmptyArray => {
            e.token(TokenKind::IDENT("EMPTY".to_string()));
            e.space();
            e.token(TokenKind::ARRAY_KW);
        }
        JsonBehaviorType::JsonBehaviorEmptyObject => {
            e.token(TokenKind::IDENT("EMPTY".to_string()));
            e.space();
            e.token(TokenKind::OBJECT_KW);
        }
        JsonBehaviorType::JsonBehaviorDefault => {
            e.token(TokenKind::DEFAULT_KW);
            if let Some(expr) = behavior.expr.as_ref() {
                e.space();
                super::emit_node(expr, e);
            } else {
                debug_assert!(false, "DEFAULT json behavior requires an expression");
            }
        }
        JsonBehaviorType::Undefined => {
            debug_assert!(false, "Undefined JSON behavior encountered");
        }
    }
}

enum JsonBehaviorClause {
    OnEmpty,
    OnError,
}

fn emit_json_behavior_clause(
    e: &mut EventEmitter,
    behavior: &JsonBehavior,
    clause: JsonBehaviorClause,
) {
    emit_json_behavior(e, behavior);
    e.space();
    e.token(TokenKind::ON_KW);
    e.space();
    match clause {
        JsonBehaviorClause::OnEmpty => e.token(TokenKind::IDENT("EMPTY".to_string())),
        JsonBehaviorClause::OnError => e.token(TokenKind::IDENT("ERROR".to_string())),
    }
}
