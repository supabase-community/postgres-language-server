//! The SET list of UPDATE, ON CONFLICT DO UPDATE and MERGE, `a = 1, (b, c) = (...)`, and the
//! target columns of INSERT. Both are lists of `ResTarget`s naming a column.

use pgls_query::protobuf::{CoercionForm, MultiAssignRef, ResTarget};

use crate::nodes::prelude::*;

pub(crate) fn emit_set_clause(e: &mut EventEmitter, n: &ResTarget) {
    // The SET list emits its ResTargets itself rather than through emit_node, so this is the only
    // place that can emit the comments written next to an assignment.
    emit_with_comments_at(e, n.location, |e| {
        e.group_start();

        if !n.name.is_empty() {
            emit_column_name_with_indirection(e, n);

            if let Some(ref val) = n.val {
                e.space();
                e.token(TokenKind::IDENT("=".to_string()));
                e.space();
                emit_node(e, val);
            }
        }

        e.group_end();
    });
}

pub(crate) fn emit_set_clause_list(e: &mut EventEmitter, nodes: &[pgls_query::Node]) {
    let mut index = 0;

    while index < nodes.len() {
        if index > 0 {
            e.token(TokenKind::COMMA);
            e.line(LineType::SoftOrSpace);
        }

        let node = &nodes[index];
        let Some(NodeEnum::ResTarget(target)) = node.node.as_ref() else {
            emit_node(e, node);
            index += 1;
            continue;
        };

        let consumed = if let Some(ref val) = target.val {
            match val.node.as_ref() {
                Some(NodeEnum::MultiAssignRef(multi)) if multi.colno == 1 => {
                    match multi.source.as_deref() {
                        Some(source) => emit_multi_assign_clause(e, nodes, index, multi, source),
                        None => {
                            emit_set_clause(e, target);
                            1
                        }
                    }
                }
                _ => {
                    emit_set_clause(e, target);
                    1
                }
            }
        } else {
            emit_set_clause(e, target);
            1
        };

        index += consumed;
    }
}

fn emit_multi_assign_clause(
    e: &mut EventEmitter,
    nodes: &[pgls_query::Node],
    start: usize,
    multi: &MultiAssignRef,
    source_node: &Node,
) -> usize {
    let total = multi.ncolumns.max(1) as usize;
    debug_assert_eq!(multi.colno, 1, "MultiAssignRef should start at colno 1");

    // The columns of `(a, b) = ...` are the next `ncolumns` targets. A source spanning past the
    // list writes the targets there are, and the AST check refuses the output.
    let end = (start + total).min(nodes.len());

    // Check if source is a RowExpr - if so, we may expand it; otherwise emit as-is
    let maybe_row_expr = if let Some(pgls_query::NodeEnum::RowExpr(r)) = source_node.node.as_ref() {
        Some(r)
    } else {
        None
    };

    let expand_tuple = maybe_row_expr
        .map(|row_expr| {
            row_expr.args.len() == total
                && matches!(row_expr.row_format(), CoercionForm::CoerceImplicitCast)
        })
        .unwrap_or(false);

    e.group_start();

    // Target columns - wrap in group for compact formatting
    e.group_start();
    e.token(TokenKind::L_PAREN);
    e.line(LineType::Soft);
    e.indent_start();
    for (idx, node) in nodes[start..end].iter().enumerate() {
        if idx > 0 {
            e.token(TokenKind::COMMA);
            e.line(LineType::SoftOrSpace);
        }

        match node.node.as_ref() {
            Some(NodeEnum::ResTarget(target)) => emit_column_name_with_indirection(e, target),
            _ => emit_node(e, node),
        }
    }
    e.indent_end();
    e.line(LineType::Soft);
    e.token(TokenKind::R_PAREN);
    e.group_end();

    e.space();
    e.token(TokenKind::IDENT("=".to_string()));
    e.space();

    // Source expressions - wrap in group for compact formatting
    if expand_tuple {
        if let Some(row_expr) = maybe_row_expr {
            e.group_start();
            e.token(TokenKind::L_PAREN);
            e.line(LineType::Soft);
            e.indent_start();
            for (idx, expr) in row_expr.args.iter().enumerate() {
                if idx > 0 {
                    e.token(TokenKind::COMMA);
                    e.line(LineType::SoftOrSpace);
                }
                emit_node(e, expr);
            }
            e.indent_end();
            e.line(LineType::Soft);
            e.token(TokenKind::R_PAREN);
            e.group_end();
        }
    } else {
        emit_node(e, source_node);
    }

    e.group_end();

    total
}

pub(crate) fn emit_column_name_with_indirection(e: &mut EventEmitter, n: &ResTarget) {
    if n.name.is_empty() {
        return;
    }

    emit_identifier_maybe_quoted(e, &n.name);

    for i in &n.indirection {
        match &i.node {
            // Field selection - emit dot before the field name
            Some(pgls_query::NodeEnum::String(s)) => {
                e.token(TokenKind::DOT);
                emit_string_identifier(e, s);
            }
            Some(n) => emit_node_enum(e, n),
            None => {}
        }
    }
}

// Emit column name only (for INSERT column list)
pub(crate) fn emit_column_name(e: &mut EventEmitter, n: &ResTarget) {
    // The INSERT column list emits its ResTargets itself rather than through emit_node, so this is
    // the only place that can emit the comments written next to a column name.
    emit_with_comments_at(e, n.location, |e| {
        e.group_start();
        emit_column_name_with_indirection(e, n);
        e.group_end();
    });
}
