use pgls_query::{
    NodeEnum,
    protobuf::{InsertStmt, SelectStmt, SetOperation},
};

use super::{
    Resolver,
    on_conflict_clause::resolve_on_conflict_clause,
    range_var::resolve_target_relation,
    res_target::{check_target_column, resolve_returning_list},
    resolve_node,
    with_clause::resolve_with_clause,
};
use crate::resolve::{
    FindingKind,
    expr::{check_assignment, infer_expr},
    scope::{Columns, Level},
};
use crate::typing::{Type, TypedColumn};

/// Resolves an INSERT and returns the columns of its RETURNING list.
pub(super) fn resolve_insert_stmt(r: &mut Resolver, n: &InsertStmt) -> Columns {
    let ctes = r.ctes.len();
    if let Some(with) = &n.with_clause {
        resolve_with_clause(r, with);
    }
    let columns = resolve_insert(r, n);
    r.ctes.truncate(ctes);
    columns
}

fn resolve_insert(r: &mut Resolver, n: &InsertStmt) -> Columns {
    let Some(relation) = &n.relation else {
        r.depends_on_file();
        return None;
    };
    let target = resolve_target_relation(r, relation);

    let mut target_columns = Vec::new();
    // Targets with subscripts or fields, like `a[1]` or `c.f`, assign to part of a column.
    let mut partial_targets = Vec::new();
    for column in &n.cols {
        let Some(NodeEnum::ResTarget(column)) = &column.node else {
            r.depends_on_file();
            continue;
        };
        check_target_column(r, &target, &column.name, column.location);
        target_columns.push(column.name.clone());
        partial_targets.push(!column.indirection.is_empty());
    }

    if let Some(source) = n.select_stmt.as_deref() {
        resolve_node(r, source);
        let source_types = r.output.clone();

        // The type of each target column, in the order of the values.
        let targets = target.typed_columns.as_deref().unwrap_or_default();
        let target_types: Vec<Option<&TypedColumn>> = if target_columns.is_empty() {
            targets.iter().map(Some).collect()
        } else {
            target_columns
                .iter()
                .zip(&partial_targets)
                .map(|(name, partial)| {
                    (!partial)
                        .then(|| targets.iter().find(|column| column.name == *name))
                        .flatten()
                })
                .collect()
        };
        let check = |r: &mut Resolver, value: Option<Type>, target: Option<&TypedColumn>| {
            if let Some(target) = target
                && let Some(column_type) = target.ty.clone()
            {
                check_assignment(r, &target.name, column_type, value, -1);
            }
        };
        match values_rows(source) {
            // Each row of `INSERT ... VALUES` is coerced to the target columns on its own.
            Some(rows) => {
                for row in rows {
                    for (value, target) in row.iter().zip(&target_types) {
                        let Some(value) = value.node.as_ref() else {
                            continue;
                        };
                        let value = infer_expr(r, value);
                        check(r, value, *target);
                    }
                }
            }
            None => {
                for (value, target) in source_types.unwrap_or_default().iter().zip(&target_types) {
                    check(r, value.ty.clone(), *target);
                }
            }
        }
        let found = match &source.node {
            Some(NodeEnum::SelectStmt(select)) => select_width(select),
            _ => None,
        };
        let expected = if target_columns.is_empty() {
            target.columns.as_ref().map(Vec::len)
        } else {
            Some(target_columns.len())
        };
        if let (Some(found), Some(expected)) = (found, expected) {
            // Without a column list, fewer values than columns are fine.
            let mismatch = if target_columns.is_empty() {
                found > expected
            } else {
                found != expected
            };
            if mismatch {
                r.report(FindingKind::InsertColumnMismatch { expected, found }, -1);
            }
        }
    }

    if let Some(conflict) = &n.on_conflict_clause {
        resolve_on_conflict_clause(r, conflict, &target);
    }

    resolve_returning_list(r, &n.returning_list, Level::with_item(target))
}

/// The rows of a plain `VALUES` list.
fn values_rows(source: &pgls_query::Node) -> Option<Vec<&[pgls_query::Node]>> {
    let Some(NodeEnum::SelectStmt(select)) = &source.node else {
        return None;
    };
    if select.op() != SetOperation::SetopNone || select.values_lists.is_empty() {
        return None;
    }
    select
        .values_lists
        .iter()
        .map(|row| match &row.node {
            Some(NodeEnum::List(row)) => Some(row.items.as_slice()),
            _ => None,
        })
        .collect()
}

/// The number of columns a query produces, if it is certain.
fn select_width(select: &SelectStmt) -> Option<usize> {
    if select.op() != SetOperation::SetopNone {
        return select_width(select.larg.as_deref()?);
    }
    if let Some(row) = select.values_lists.first() {
        return match &row.node {
            Some(NodeEnum::List(row)) => Some(row.items.len()),
            _ => None,
        };
    }
    let mut width = 0;
    for target in &select.target_list {
        let NodeEnum::ResTarget(target) = target.node.as_ref()? else {
            return None;
        };
        if expands_to_columns(target.val.as_deref()?.node.as_ref()?) {
            return None;
        }
        width += 1;
    }
    Some(width)
}

/// `*`, `t.*`, and `(row).*` expand to any number of columns.
fn expands_to_columns(value: &NodeEnum) -> bool {
    let last = match value {
        NodeEnum::ColumnRef(column) => column.fields.last(),
        NodeEnum::AIndirection(indirection) => indirection.indirection.last(),
        _ => return false,
    };
    last.is_some_and(|last| matches!(last.node, Some(NodeEnum::AStar(_))))
}
