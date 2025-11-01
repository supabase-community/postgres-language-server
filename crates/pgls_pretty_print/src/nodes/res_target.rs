use pgls_query::{
    NodeEnum,
    protobuf::{CoercionForm, MultiAssignRef, ResTarget},
};

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
};

use super::{emit_identifier_maybe_quoted, emit_node};

pub(super) fn emit_res_target(e: &mut EventEmitter, n: &ResTarget) {
    e.group_start(GroupKind::ResTarget);

    if let Some(ref val) = n.val {
        emit_node(val, e);

        if !n.name.is_empty() {
            e.space();
            e.token(TokenKind::AS_KW);
            e.space();
            emit_identifier_maybe_quoted(e, &n.name);
        }
    }

    e.group_end();
}

pub(super) fn emit_set_clause(e: &mut EventEmitter, n: &ResTarget) {
    e.group_start(GroupKind::ResTarget);

    if !n.name.is_empty() {
        emit_column_name_with_indirection(e, n);

        if let Some(ref val) = n.val {
            e.space();
            e.token(TokenKind::IDENT("=".to_string()));
            e.space();
            emit_node(val, e);
        }
    }

    e.group_end();
}

pub(super) fn emit_set_clause_list(e: &mut EventEmitter, nodes: &[pgls_query::Node]) {
    let mut index = 0;

    while index < nodes.len() {
        if index > 0 {
            e.token(TokenKind::COMMA);
            e.line(LineType::SoftOrSpace);
        }

        let node = &nodes[index];
        let target = assert_node_variant!(ResTarget, node);

        let consumed = if let Some(ref val) = target.val {
            match val.node.as_ref() {
                Some(NodeEnum::MultiAssignRef(multi)) if multi.colno == 1 => {
                    emit_multi_assign_clause(e, nodes, index, multi)
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
) -> usize {
    let total = multi.ncolumns.max(1) as usize;
    debug_assert_eq!(multi.colno, 1, "MultiAssignRef should start at colno 1");

    let end = start + total;
    debug_assert!(
        end <= nodes.len(),
        "MultiAssignRef spans beyond target list"
    );

    let source_node = multi
        .source
        .as_ref()
        .expect("MultiAssignRef source missing row expression");
    let row_expr = assert_node_variant!(RowExpr, source_node);

    let expand_tuple = row_expr.args.len() == total
        && matches!(row_expr.row_format(), CoercionForm::CoerceImplicitCast);

    e.group_start(GroupKind::ResTarget);

    // Target columns
    e.token(TokenKind::L_PAREN);
    for (idx, node) in nodes[start..end].iter().enumerate() {
        if idx > 0 {
            e.token(TokenKind::COMMA);
            e.line(LineType::SoftOrSpace);
        }

        let target = assert_node_variant!(ResTarget, node);
        emit_column_name_with_indirection(e, target);
    }
    e.token(TokenKind::R_PAREN);

    e.space();
    e.token(TokenKind::IDENT("=".to_string()));
    e.space();

    // Source expressions
    if expand_tuple {
        e.token(TokenKind::L_PAREN);
        for (idx, expr) in row_expr.args.iter().enumerate() {
            if idx > 0 {
                e.token(TokenKind::COMMA);
                e.line(LineType::SoftOrSpace);
            }
            emit_node(expr, e);
        }
        e.token(TokenKind::R_PAREN);
    } else {
        emit_node(source_node, e);
    }

    e.group_end();

    total
}

pub(super) fn emit_column_name_with_indirection(e: &mut EventEmitter, n: &ResTarget) {
    if n.name.is_empty() {
        return;
    }

    e.token(TokenKind::IDENT(n.name.clone()));

    for i in &n.indirection {
        match &i.node {
            // Field selection - emit dot before the field name
            Some(pgls_query::NodeEnum::String(s)) => {
                e.token(TokenKind::DOT);
                super::emit_string_identifier(e, s);
            }
            Some(n) => super::emit_node_enum(n, e),
            None => {}
        }
    }
}

// Emit column name only (for INSERT column list)
pub(super) fn emit_column_name(e: &mut EventEmitter, n: &ResTarget) {
    e.group_start(GroupKind::ResTarget);
    emit_column_name_with_indirection(e, n);
    e.group_end();
}
