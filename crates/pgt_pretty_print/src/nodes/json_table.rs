use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
    nodes::node_list::emit_comma_separated_list,
};
use pgt_query::{NodeEnum, protobuf::JsonTable};

pub(super) fn emit_json_table(e: &mut EventEmitter, n: &JsonTable) {
    e.group_start(GroupKind::JsonTable);

    e.token(TokenKind::IDENT("JSON_TABLE".to_string()));
    e.token(TokenKind::L_PAREN);

    // Context item (the JSON data)
    if let Some(ref context) = n.context_item {
        if let Some(ref raw_expr) = context.raw_expr {
            super::emit_node(raw_expr, e);
        }
    }

    e.token(TokenKind::COMMA);
    e.space();

    // Path specification
    if let Some(ref pathspec) = n.pathspec {
        if let Some(ref string_node) = pathspec.string {
            super::emit_node(string_node, e);
        }
    }

    // PASSING clause
    if !n.passing.is_empty() {
        e.space();
        e.token(TokenKind::IDENT("PASSING".to_string()));
        e.space();
        emit_comma_separated_list(e, &n.passing, super::emit_node);
    }

    // COLUMNS clause
    e.space();
    e.token(TokenKind::IDENT("COLUMNS".to_string()));
    e.space();
    e.token(TokenKind::L_PAREN);

    if !n.columns.is_empty() {
        emit_comma_separated_list(e, &n.columns, |node, e| {
            if let Some(NodeEnum::JsonTableColumn(col)) = &node.node {
                // Column name
                e.token(TokenKind::IDENT(col.name.clone()));

                // Column type (regular, ordinality, exists, query, etc.)
                // For now, emit type name for regular columns
                if let Some(ref type_name) = col.type_name {
                    e.space();
                    super::emit_type_name(e, type_name);
                }

                // Path specification for the column
                if let Some(ref pathspec) = col.pathspec {
                    e.space();
                    e.token(TokenKind::IDENT("PATH".to_string()));
                    e.space();
                    if let Some(ref string_node) = pathspec.string {
                        super::emit_node(string_node, e);
                    }
                }

                // TODO: Handle ON EMPTY, ON ERROR, nested columns
            }
        });
    }

    e.token(TokenKind::R_PAREN);
    e.token(TokenKind::R_PAREN);

    // Alias (emit_alias includes the AS keyword)
    if let Some(ref alias) = n.alias {
        e.space();
        super::emit_alias(e, alias);
    }

    e.group_end();
}
