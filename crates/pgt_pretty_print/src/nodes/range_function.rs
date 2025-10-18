use pgt_query::protobuf::RangeFunction;

use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind};

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
            if let Some(pgt_query::NodeEnum::List(func_list)) = node.node.as_ref() {
                if !func_list.items.is_empty() {
                    // Emit the function call (first item)
                    super::emit_node(&func_list.items[0], e);

                    // Emit column definitions if present (items after first)
                    if func_list.items.len() > 1 {
                        e.space();
                        e.token(TokenKind::AS_KW);
                        e.space();
                        e.token(TokenKind::L_PAREN);
                        emit_comma_separated_list(e, &func_list.items[1..], super::emit_node);
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
            if let Some(pgt_query::NodeEnum::List(func_list)) = n.functions[0].node.as_ref() {
                if !func_list.items.is_empty() {
                    super::emit_node(&func_list.items[0], e);
                }
            } else {
                super::emit_node(&n.functions[0], e);
            }
        }
    }

    if n.ordinality {
        e.space();
        e.token(TokenKind::WITH_KW);
        e.space();
        e.token(TokenKind::ORDINALITY_KW);
    }

    if let Some(ref alias) = n.alias {
        e.space();
        super::emit_alias(e, alias);
    }

    e.group_end();
}
