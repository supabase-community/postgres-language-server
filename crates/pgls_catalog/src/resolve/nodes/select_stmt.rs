//! SELECT, set operations, and VALUES.

use pgls_query::{
    Node, NodeEnum,
    protobuf::{SelectStmt, SetOperation},
};

use super::{
    Resolver, from_clause::resolve_from_clause, res_target::target_list_columns, resolve_list,
    resolve_node, with_clause::resolve_with_clause,
};
use crate::column_name::figure_column_name;
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

/// Set operations take their column names from the left-most query. Their ORDER BY and LIMIT
/// reference the output columns and are not checked.
fn resolve_set_operation(r: &mut Resolver, n: &SelectStmt) -> Columns {
    let columns = n
        .larg
        .as_deref()
        .and_then(|left| resolve_select_stmt(r, left));
    if let Some(right) = n.rarg.as_deref() {
        resolve_select_stmt(r, right);
    }
    columns
}

fn resolve_values(r: &mut Resolver, n: &SelectStmt) -> Columns {
    let mut width = None;
    for row in &n.values_lists {
        if let Some(NodeEnum::List(row)) = &row.node {
            width.get_or_insert(row.items.len());
            resolve_list(r, &row.items);
        }
    }
    width.map(|width| (1..=width).map(|i| format!("column{i}")).collect())
}

fn resolve_select(r: &mut Resolver, n: &SelectStmt) -> Columns {
    let mut level = resolve_from_clause(r, &n.from_clause);
    level.output_names = n.target_list.iter().filter_map(output_name).collect();
    r.enter_level(level);

    resolve_list(r, &n.target_list);
    for node in n.where_clause.iter().chain(&n.having_clause) {
        resolve_node(r, node);
    }
    r.set_output_names_visible(true);
    resolve_list(r, &n.group_clause);
    resolve_list(r, &n.sort_clause);
    resolve_list(r, &n.distinct_clause);
    r.set_output_names_visible(false);
    resolve_list(r, &n.window_clause);
    for node in n.limit_offset.iter().chain(&n.limit_count) {
        resolve_node(r, node);
    }

    let columns = target_list_columns(r, &n.target_list);
    r.exit_level();
    columns
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
