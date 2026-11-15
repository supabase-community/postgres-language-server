//! SELECT, set operations, and VALUES.

use pgls_query::{
    Node, NodeEnum,
    protobuf::{CoercionForm, SelectStmt, SetOperation},
};
use pgls_query_ext::utils::string_value;

use super::{
    Resolver,
    from_clause::resolve_from_clause,
    record_unresolved_node,
    res_target::{infer_select_list, record_star_references, select_list_output},
    resolve_list, resolve_node,
    with_clause::resolve_with_clause,
};
use crate::resolve::{
    expr::infer_expr,
    lookups::record_sort_by,
    output::{QueryOutput, TargetOutput},
    scope::OutputNames,
    utils::{node_location, resolve_expr},
};
use crate::typing::TypedColumn;

#[cfg(test)]
thread_local! {
    /// How many queries this thread resolved, for tests of how much work a statement takes.
    pub(crate) static RESOLVED_QUERIES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Resolves a query and returns its output columns.
pub(super) fn resolve_select_stmt(r: &mut Resolver, n: &SelectStmt) -> Option<QueryOutput> {
    #[cfg(test)]
    RESOLVED_QUERIES.with(|count| count.set(count.get() + 1));
    let ctes = r.ctes.len();
    if let Some(with) = &n.with_clause {
        resolve_with_clause(r, with);
    }

    let output = if n.op() != SetOperation::SetopNone {
        resolve_set_operation(r, n)
    } else if !n.values_lists.is_empty() {
        resolve_values(r, n)
    } else {
        resolve_select(r, n)
    };

    r.ctes.truncate(ctes);
    output
}

/// Set operations take their column names from the left-most query and the common type of each
/// column pair. Their ORDER BY and LIMIT reference the output columns and are not checked. Port
/// of [`transformSetOperationStmt`] and [`transformSetOperationTree`].
///
/// [`transformSetOperationStmt`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/analyze.c#L1751
/// [`transformSetOperationTree`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/analyze.c#L2056
fn resolve_set_operation(r: &mut Resolver, n: &SelectStmt) -> Option<QueryOutput> {
    let left = n
        .larg
        .as_deref()
        .and_then(|left| resolve_select_stmt(r, left));
    let right = n
        .rarg
        .as_deref()
        .and_then(|right| resolve_select_stmt(r, right));
    // ORDER BY and LIMIT see only the output columns.
    let clauses = n
        .sort_clause
        .iter()
        .chain(n.limit_offset.as_deref())
        .chain(n.limit_count.as_deref());
    for node in clauses {
        record_unresolved_node(r, node, false);
    }
    let mut output = left?;
    let left = output.columns();
    let right = right.as_ref().and_then(QueryOutput::columns);
    match (left, right) {
        (Some(left), Some(right)) if left.len() == right.len() => {
            let types: Vec<_> = left
                .into_iter()
                .zip(right)
                .map(|(left, right)| match (left.ty, right.ty) {
                    (Some(left), Some(right)) => {
                        match crate::typing::select_common_type(r.catalog, &[left, right]) {
                            crate::typing::Selection::Match(ty) => Some(ty),
                            _ => None,
                        }
                    }
                    _ => None,
                })
                .collect();
            output.set_types(types);
        }
        _ => output.types_known = false,
    }
    Some(output)
}

/// `VALUES` names its columns `column1`, `column2`, ..., and types each as the common type of
/// its rows. Port of [`transformValuesClause`].
///
/// [`transformValuesClause`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/analyze.c#L1532
fn resolve_values(r: &mut Resolver, n: &SelectStmt) -> Option<QueryOutput> {
    let mut first_row = None;
    for row in &n.values_lists {
        if let Some(NodeEnum::List(row)) = &row.node {
            first_row.get_or_insert(row);
            resolve_list(r, &row.items);
        }
    }
    let first_row = first_row?;
    let targets = first_row
        .items
        .iter()
        .enumerate()
        .map(|(index, first)| {
            let values = n
                .values_lists
                .iter()
                .filter_map(|row| row.node.as_ref())
                .filter_map(|row| match row {
                    NodeEnum::List(row) => row.items.get(index),
                    _ => None,
                })
                .map(|node| node.node.as_ref().and_then(|expr| infer_expr(r, expr)))
                .collect::<Option<Vec<_>>>();
            let ty = values.and_then(|values| {
                match crate::typing::select_common_type(r.catalog, &values) {
                    crate::typing::Selection::Match(ty) => Some(ty),
                    _ => None,
                }
            });
            TargetOutput {
                location: node_location(first).unwrap_or(-1),
                star: None,
                columns: Some(vec![TypedColumn {
                    name: format!("column{}", index + 1),
                    ty,
                }]),
            }
        })
        .collect();
    Some(QueryOutput::new(targets))
}

/// A plain SELECT: its FROM clause, select list, WHERE, GROUP BY, HAVING, window and ORDER BY
/// clauses, in the order of [`transformSelectStmt`].
///
/// [`transformSelectStmt`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/analyze.c#L1389
fn resolve_select(r: &mut Resolver, n: &SelectStmt) -> Option<QueryOutput> {
    let level = resolve_from_clause(r, &n.from_clause);
    r.enter_level(level);

    resolve_list(r, &n.target_list);
    // Named after resolving the select list, which names its scalar subqueries.
    let mut output = select_list_output(r, &n.target_list);
    let output_names = output.output_names();
    if let Some(level) = r.levels.last_mut() {
        level.output_names = output_names.clone();
    }
    record_star_references(r, &output);
    for node in n.where_clause.iter().chain(&n.having_clause) {
        resolve_expr(r, node);
    }
    // In `GROUP BY`, a column of FROM takes precedence over an output column with its name,
    // unlike in `ORDER BY` and `DISTINCT ON`.
    resolve_clause_items(r, &n.group_clause, OutputNames::AfterColumns, true);
    for node in &n.group_clause {
        if let Some(expr) = node.node.as_ref()
            && !is_output_reference(expr, &output_names)
        {
            infer_expr(r, expr);
        }
    }
    resolve_clause_items(r, &n.sort_clause, OutputNames::BeforeColumns, false);
    for node in &n.sort_clause {
        if let Some(expr) = node.node.as_ref().and_then(|sort| match sort {
            NodeEnum::SortBy(sort) => sort.node.as_deref().and_then(|node| node.node.as_ref()),
            _ => Some(sort),
        }) && !is_output_reference(expr, &output_names)
        {
            infer_expr(r, expr);
        }
    }
    resolve_clause_items(r, &n.distinct_clause, OutputNames::BeforeColumns, false);
    for node in &n.distinct_clause {
        if let Some(expr) = node.node.as_ref()
            && !is_output_reference(expr, &output_names)
        {
            infer_expr(r, expr);
        }
    }
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
                    infer_expr(r, expr);
                }
            }
        }
    }
    for node in n.limit_offset.iter().chain(&n.limit_count) {
        resolve_expr(r, node);
    }

    // PR 3: Postgres types the select list before the other clauses (`transformTargetList`).
    // Typing it last keeps the order of findings and references.
    infer_select_list(r, &n.target_list, &mut output);
    r.exit_level();
    Some(output)
}

/// Resolves the items of `GROUP BY`, `ORDER BY` or `DISTINCT ON`, where an item that is a bare
/// name can be an output column, with the precedence of `output_names` ([`findTargetlistEntrySQL92`]).
/// The names in an expression are columns, like elsewhere ([`findTargetlistEntrySQL99`]). In
/// `GROUP BY` (`grouping`), the items of grouping sets and of parenthesized lists are items of
/// their own ([`flatten_grouping_sets`]). Every node of the clause is visited once, without
/// recursion.
///
/// [`findTargetlistEntrySQL92`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_clause.c#L2006
/// [`findTargetlistEntrySQL99`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_clause.c#L2172
/// [`flatten_grouping_sets`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_clause.c#L2258
fn resolve_clause_items(
    r: &mut Resolver,
    items: &[Node],
    output_names: OutputNames,
    grouping: bool,
) {
    let mut pending: Vec<&Node> = items.iter().rev().collect();
    while let Some(node) = pending.pop() {
        let item = match node.node.as_ref() {
            Some(NodeEnum::GroupingSet(set)) if grouping => {
                pending.extend(set.content.iter().rev());
                continue;
            }
            Some(NodeEnum::RowExpr(row))
                if grouping && row.row_format() == CoercionForm::CoerceImplicitCast =>
            {
                pending.extend(row.args.iter().rev());
                continue;
            }
            Some(NodeEnum::SortBy(sort)) => {
                record_sort_by(r, sort);
                match sort.node.as_deref() {
                    Some(item) => item,
                    None => continue,
                }
            }
            _ => node,
        };
        let is_bare_name = matches!(
            item.node.as_ref(),
            Some(NodeEnum::ColumnRef(column)) if matches!(column.fields.as_slice(), [field] if string_value(field).is_some())
        );
        if is_bare_name {
            r.set_output_names_visible(output_names);
        }
        resolve_node(r, item);
        r.set_output_names_visible(OutputNames::Hidden);
    }
}

/// Bare output names and ordinal references are resolved by Postgres against the target list
/// ([`findTargetlistEntrySQL92`]).
///
/// [`findTargetlistEntrySQL92`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_clause.c#L2006
fn is_output_reference(expr: &NodeEnum, output_names: &[String]) -> bool {
    match expr {
        NodeEnum::ColumnRef(column) => match column.fields.as_slice() {
            [field] => string_value(field)
                .is_some_and(|name| output_names.iter().any(|output| output == name)),
            _ => false,
        },
        NodeEnum::AConst(value) => value
            .val
            .as_ref()
            .is_some_and(|value| matches!(value, pgls_query::protobuf::a_const::Val::Ival(_))),
        _ => false,
    }
}
