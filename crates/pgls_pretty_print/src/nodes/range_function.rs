use pgls_query::protobuf::RangeFunction;

use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind, LineType};

use super::node_list::emit_comma_separated_list;

pub(super) fn emit_range_function(e: &mut EventEmitter, n: &RangeFunction) {
    e.group_start(GroupKind::RangeFunction);

    if n.lateral {
        e.token(TokenKind::LATERAL_KW);
        e.space();
    }

    if n.is_rowsfrom {
        e.token(TokenKind::ROWS_KW);
        e.space();
        e.token(TokenKind::FROM_KW);
        e.space();
        e.token(TokenKind::L_PAREN);

        emit_comma_separated_list(e, &n.functions, |node, e| {
            // Each item is a List containing function + optional column definitions
            if let Some(pgls_query::NodeEnum::List(func_list)) = node.node.as_ref() {
                if !func_list.items.is_empty() {
                    // Emit the function call (first item)
                    super::emit_node(&func_list.items[0], e);

                    // Emit column definitions if present (items after first)
                    // Check that there are actual non-empty column definitions
                    let col_defs: Vec<pgls_query::Node> = func_list.items[1..]
                        .iter()
                        .filter(|item| item.node.is_some())
                        .cloned()
                        .collect();
                    if !col_defs.is_empty() {
                        e.space();
                        e.token(TokenKind::AS_KW);
                        e.space();
                        e.token(TokenKind::L_PAREN);
                        emit_comma_separated_list(e, &col_defs, super::emit_node);
                        e.token(TokenKind::R_PAREN);
                    }
                }
            } else {
                super::emit_node(node, e);
            }
        });

        e.token(TokenKind::R_PAREN);
    } else {
        // Simple function call - Functions contains a single List with one function
        if !n.functions.is_empty() {
            // For non-ROWS FROM, functions[0] is the List containing the function
            if let Some(pgls_query::NodeEnum::List(func_list)) = n.functions[0].node.as_ref() {
                if !func_list.items.is_empty() {
                    super::emit_node(&func_list.items[0], e);
                }
            } else {
                super::emit_node(&n.functions[0], e);
            }
        }
    }

    if n.ordinality {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::WITH_KW);
        e.space();
        e.token(TokenKind::ORDINALITY_KW);
    }

    // Handle alias and coldeflist
    // For RangeFunction, when we have both alias and coldeflist:
    // Correct syntax: func(...) AS aliasname (col1 type1, col2 type2, ...)
    if let Some(ref alias) = n.alias {
        if !n.coldeflist.is_empty() {
            // Has both alias and coldeflist - emit alias name first, then coldeflist
            e.line(LineType::SoftOrSpace);
            e.token(TokenKind::AS_KW);
            e.space();
            super::emit_identifier_maybe_quoted(e, &alias.aliasname);
            // Allow line break before column definition list for narrow widths
            e.line(LineType::SoftOrSpace);
            e.group_start(GroupKind::List);
            e.token(TokenKind::L_PAREN);
            e.line(LineType::Soft);
            e.indent_start();
            emit_comma_separated_list(e, &n.coldeflist, super::emit_node);
            e.indent_end();
            e.line(LineType::Soft);
            e.token(TokenKind::R_PAREN);
            e.group_end();
        } else {
            // Just alias with potential colnames - use emit_alias
            e.line(LineType::SoftOrSpace);
            super::emit_alias(e, alias);
        }
    } else if !n.coldeflist.is_empty() {
        // No alias but has coldeflist - emit with AS
        e.space();
        e.token(TokenKind::AS_KW);
        e.line(LineType::SoftOrSpace);
        e.group_start(GroupKind::List);
        e.token(TokenKind::L_PAREN);
        e.line(LineType::Soft);
        e.indent_start();
        emit_comma_separated_list(e, &n.coldeflist, super::emit_node);
        e.indent_end();
        e.line(LineType::Soft);
        e.token(TokenKind::R_PAREN);
        e.group_end();
    }

    e.group_end();
}
