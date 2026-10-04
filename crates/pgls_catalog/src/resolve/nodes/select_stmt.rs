//! SELECT, set operations, and VALUES.

use pgls_query::{
    Node, NodeEnum,
    protobuf::{SelectStmt, SetOperation},
};

use super::{
    Resolver, from_clause::resolve_from_clause, res_target::target_list_columns, resolve_list,
    resolve_node, with_clause::resolve_with_clause,
};
use crate::resolve::column_name::figure_column_name;
use crate::resolve::scope::Columns;

/// Resolves a query and returns its output columns, if known.
pub(super) fn resolve_select_stmt(r: &mut Resolver, n: &SelectStmt) -> Columns {
    let ctes = r.ctes.len();
    if let Some(with) = &n.with_clause {
        resolve_with_clause(r, with);
    }

    let columns = if n.op() != SetOperation::SetopNone {
        resolve_set_operation(r, n)
    } else if !n.values_lists.is_empty() {
        resolve_values(r, n)
    } else {
        resolve_select(r, n)
    };

    r.ctes.truncate(ctes);
    columns
}

/// Set operations take their column names from the left-most query and the common type of each
/// column pair. Their ORDER BY and LIMIT reference the output columns and are not checked. Port
/// of [`transformSetOperationStmt`] and [`transformSetOperationTree`].
///
/// [`transformSetOperationStmt`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/analyze.c#L1751
/// [`transformSetOperationTree`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/analyze.c#L2056
fn resolve_set_operation(r: &mut Resolver, n: &SelectStmt) -> Columns {
    let columns = n
        .larg
        .as_deref()
        .and_then(|left| resolve_select_stmt(r, left));
    let left_typed = r.output.clone();
    if let Some(right) = n.rarg.as_deref() {
        resolve_select_stmt(r, right);
    }
    let right_typed = r.output.clone();
    r.output = match (left_typed, right_typed) {
        (Some(left), Some(right)) if left.len() == right.len() => Some(
            left.into_iter()
                .zip(right)
                .map(|(left, right)| {
                    let ty = match (left.ty, right.ty) {
                        (Some(left), Some(right)) => {
                            match crate::typing::select_common_type(r.catalog, &[left, right]) {
                                crate::typing::Selection::Match(ty) => Some(ty),
                                _ => None,
                            }
                        }
                        _ => None,
                    };
                    crate::typing::TypedColumn {
                        name: left.name,
                        ty,
                    }
                })
                .collect(),
        ),
        _ => None,
    };
    columns
}

/// `VALUES` names its columns `column1`, `column2`, ..., and types each as the common type of
/// its rows. Port of [`transformValuesClause`].
///
/// [`transformValuesClause`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/analyze.c#L1532
fn resolve_values(r: &mut Resolver, n: &SelectStmt) -> Columns {
    let mut width = None;
    for row in &n.values_lists {
        if let Some(NodeEnum::List(row)) = &row.node {
            width.get_or_insert(row.items.len());
            resolve_list(r, &row.items);
        }
    }
    let columns = width.map(|width| {
        (1..=width)
            .map(|i| format!("column{i}"))
            .collect::<Vec<_>>()
    });
    r.output = columns.as_ref().map(|names| {
        names
            .iter()
            .enumerate()
            .map(|(index, name)| {
                let values = n
                    .values_lists
                    .iter()
                    .filter_map(|row| row.node.as_ref())
                    .filter_map(|row| match row {
                        NodeEnum::List(row) => row.items.get(index),
                        _ => None,
                    })
                    .map(|node| {
                        node.node
                            .as_ref()
                            .and_then(|expr| super::super::expr::infer_expr(r, expr))
                    })
                    .collect::<Option<Vec<_>>>();
                let ty = values.and_then(|values| {
                    match crate::typing::select_common_type(r.catalog, &values) {
                        crate::typing::Selection::Match(ty) => Some(ty),
                        _ => None,
                    }
                });
                crate::typing::TypedColumn {
                    name: name.clone(),
                    ty,
                }
            })
            .collect()
    });
    columns
}

/// A plain SELECT: its FROM clause, select list, WHERE, GROUP BY, HAVING, window and ORDER BY
/// clauses, in the order of [`transformSelectStmt`].
///
/// [`transformSelectStmt`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/analyze.c#L1389
fn resolve_select(r: &mut Resolver, n: &SelectStmt) -> Columns {
    let mut level = resolve_from_clause(r, &n.from_clause);
    level.output_names = n.target_list.iter().filter_map(output_name).collect();
    let output_names = level.output_names.clone();
    r.enter_level(level);

    resolve_list(r, &n.target_list);
    for node in n.where_clause.iter().chain(&n.having_clause) {
        resolve_node(r, node);
        if let Some(expr) = node.node.as_ref() {
            super::super::expr::infer_expr(r, expr);
        }
    }
    r.set_output_names_visible(true);
    for node in &n.group_clause {
        resolve_node(r, node);
        if let Some(expr) = node.node.as_ref()
            && !is_output_reference(expr, &output_names)
        {
            super::super::expr::infer_expr(r, expr);
        }
    }
    for node in &n.sort_clause {
        resolve_node(r, node);
        if let Some(expr) = node.node.as_ref().and_then(|sort| match sort {
            NodeEnum::SortBy(sort) => sort.node.as_deref().and_then(|node| node.node.as_ref()),
            _ => Some(sort),
        }) && !is_output_reference(expr, &output_names)
        {
            super::super::expr::infer_expr(r, expr);
        }
    }
    for node in &n.distinct_clause {
        resolve_node(r, node);
        if let Some(expr) = node.node.as_ref()
            && !is_output_reference(expr, &output_names)
        {
            super::super::expr::infer_expr(r, expr);
        }
    }
    r.set_output_names_visible(false);
    for node in &n.window_clause {
        resolve_node(r, node);
        if let Some(NodeEnum::WindowDef(window)) = node.node.as_ref() {
            for expr in window
                .partition_clause
                .iter()
                .chain(&window.order_clause)
                .chain(window.start_offset.iter().map(|offset| offset.as_ref()))
                .chain(window.end_offset.iter().map(|offset| offset.as_ref()))
            {
                if let Some(expr) = expr.node.as_ref() {
                    super::super::expr::infer_expr(r, expr);
                }
            }
        }
    }
    for node in n.limit_offset.iter().chain(&n.limit_count) {
        resolve_node(r, node);
        if let Some(expr) = node.node.as_ref() {
            super::super::expr::infer_expr(r, expr);
        }
    }

    let columns = target_list_columns(r, &n.target_list);
    let typed = columns.as_ref().map(|names| {
        let mut out = Vec::new();
        for target in &n.target_list {
            let Some(NodeEnum::ResTarget(t)) = target.node.as_ref() else {
                continue;
            };
            let Some(value) = t.val.as_deref().and_then(|v| v.node.as_ref()) else {
                continue;
            };
            if let NodeEnum::ColumnRef(c) = value
                && c.fields
                    .last()
                    .is_some_and(|f| matches!(f.node, Some(NodeEnum::AStar(_))))
            {
                let qualifier = c
                    .fields
                    .iter()
                    .rev()
                    .nth(1)
                    .and_then(|f| super::string::string_value(f));
                if let Some(item) = qualifier.and_then(|q| r.levels.last().and_then(|l| l.item(q)))
                {
                    out.extend(item.typed_columns.clone().unwrap_or_default());
                } else if let Some(level) = r.levels.last() {
                    out.extend(level.star_typed_columns().unwrap_or_default());
                }
                continue;
            }
            let name = if !t.name.is_empty() {
                t.name.clone()
            } else {
                super::super::column_name::figure_column_name(value).unwrap_or_default()
            };
            out.push(crate::typing::TypedColumn {
                name,
                ty: super::super::expr::infer_expr(r, value),
            });
        }
        if out.len() == names.len() {
            out
        } else {
            names
                .iter()
                .map(|name| crate::typing::TypedColumn {
                    name: name.clone(),
                    ty: None,
                })
                .collect()
        }
    });
    r.output = typed;
    r.exit_level();
    columns
}

/// Bare output names and ordinal references are resolved by Postgres against the target list
/// ([`findTargetlistEntrySQL92`]).
///
/// [`findTargetlistEntrySQL92`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_clause.c#L2006
fn is_output_reference(expr: &NodeEnum, output_names: &[String]) -> bool {
    match expr {
        NodeEnum::ColumnRef(column) if column.fields.len() == 1 => column.fields[0]
            .node
            .as_ref()
            .and_then(|node| match node {
                NodeEnum::String(value) => Some(value.sval.as_str()),
                _ => None,
            })
            .is_some_and(|name| output_names.iter().any(|output| output == name)),
        NodeEnum::AConst(value) => value
            .val
            .as_ref()
            .is_some_and(|value| matches!(value, pgls_query::protobuf::a_const::Val::Ival(_))),
        _ => false,
    }
}

/// The name of an entry of the select list, which `ORDER BY` and `GROUP BY` can reference.
fn output_name(target: &Node) -> Option<String> {
    match &target.node {
        Some(NodeEnum::ResTarget(target)) if !target.name.is_empty() => Some(target.name.clone()),
        Some(NodeEnum::ResTarget(target)) => target
            .val
            .as_deref()
            .and_then(|value| value.node.as_ref())
            .and_then(figure_column_name),
        _ => None,
    }
}
