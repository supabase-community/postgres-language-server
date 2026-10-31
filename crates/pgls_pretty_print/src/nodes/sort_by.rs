use pgls_query::protobuf::{SortBy, SortByDir, SortByNulls};

use super::prelude::*;

pub(super) fn emit_sort_by(e: &mut EventEmitter, n: &SortBy) {
    e.group_start();

    // Emit the expression being sorted
    if let Some(ref node) = n.node {
        emit_node(e, node);
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
        x if x == SortByDir::SortbyUsing as i32 && !n.use_op.is_empty() => {
            e.space();
            e.token(TokenKind::USING_KW);
            e.space();

            emit_operator(e, &n.use_op);
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
