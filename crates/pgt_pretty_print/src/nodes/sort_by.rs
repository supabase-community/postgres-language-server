use pgt_query::protobuf::{SortBy, SortByDir, SortByNulls};

use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind};

pub(super) fn emit_sort_by(e: &mut EventEmitter, n: &SortBy) {
    e.group_start(GroupKind::SortBy);

    // Emit the expression being sorted
    if let Some(ref node) = n.node {
        super::emit_node(node, e);
    }

    // Add sort direction
    match n.sortby_dir {
        x if x == SortByDir::SortbyAsc as i32 => {
            e.space();
            e.token(TokenKind::ASC_KW);
        }
        x if x == SortByDir::SortbyDesc as i32 => {
            e.space();
            e.token(TokenKind::DESC_KW);
        }
        x if x == SortByDir::SortbyUsing as i32 => {
            if !n.use_op.is_empty() {
                e.space();
                e.token(TokenKind::USING_KW);
                e.space();

                // Emit operator - could be qualified like schema.op
                if n.use_op.len() > 1 {
                    // Multiple parts: use OPERATOR(schema.op) syntax
                    e.token(TokenKind::OPERATOR_KW);
                    e.token(TokenKind::L_PAREN);
                    emit_operator_name(e, &n.use_op);
                    e.token(TokenKind::R_PAREN);
                } else if n.use_op.len() == 1 {
                    // Single part: use direct operator syntax
                    emit_operator_name(e, &n.use_op);
                }
            }
        }
        _ => {
            // Default - no explicit direction
        }
    }

    // Add null ordering
    match n.sortby_nulls {
        x if x == SortByNulls::SortbyNullsFirst as i32 => {
            e.space();
            e.token(TokenKind::NULLS_KW);
            e.space();
            e.token(TokenKind::FIRST_KW);
        }
        x if x == SortByNulls::SortbyNullsLast as i32 => {
            e.space();
            e.token(TokenKind::NULLS_KW);
            e.space();
            e.token(TokenKind::LAST_KW);
        }
        _ => {
            // Default - no explicit null ordering
        }
    }

    e.group_end();
}

fn emit_operator_name(e: &mut EventEmitter, use_op: &[pgt_query::protobuf::Node]) {
    for (i, node) in use_op.iter().enumerate() {
        if i > 0 {
            e.token(TokenKind::DOT);
        }

        if let Some(pgt_query::NodeEnum::String(s)) = node.node.as_ref() {
            // Operator name - emit as identifier
            e.token(TokenKind::IDENT(s.sval.clone()));
        } else {
            super::emit_node(node, e);
        }
    }
}
