use pgls_query::{
    NodeEnum,
    protobuf::{InsertStmt, OverridingKind, SelectStmt, SetOperation},
};

use super::{
    Resolver,
    on_conflict_clause::resolve_on_conflict_clause,
    range_var::resolve_target_relation,
    res_target::{check_target_column, resolve_returning_clause},
    resolve_node,
    with_clause::resolve_with_clause,
};
use crate::resolve::{
    FindingKind,
    expr::{check_assignment, infer_expr},
    scope::{Columns, Item, Level},
};
use crate::typing::{Type, TypedColumn};
use pgls_text_size::TextRange;

/// Resolves an INSERT and returns the columns of its RETURNING list. Port of
/// [`transformInsertStmt`]: the target columns ([`checkInsertTargets`]), the number of
/// values, and the assignment of each value ([`transformInsertRow`]).
///
/// [`transformInsertStmt`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/analyze.c#L633
/// [`checkInsertTargets`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_target.c#L1018
/// [`transformInsertRow`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/analyze.c#L1060
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

    // The number of columns of the source query, if known.
    let mut source_width = None;
    if let Some(source) = n.select_stmt.as_deref() {
        resolve_node(r, source);
        let source_types = r.output.clone();
        source_width = source_types.as_ref().map(Vec::len);

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
        let check = |r: &mut Resolver,
                     expr: Option<&NodeEnum>,
                     value: Option<Type>,
                     target: Option<&TypedColumn>,
                     location: i32| {
            if let Some(target) = target
                && let Some(column_type) = target.ty.clone()
            {
                check_assignment(r, &target.name, column_type, expr, value, location);
            }
        };
        match values_rows(source) {
            // Each row of `INSERT ... VALUES` is coerced to the target columns on its own.
            Some(rows) => {
                for row in rows {
                    for (value, target) in row.iter().zip(&target_types) {
                        let location = node_location(value).unwrap_or_default();
                        let Some(value) = value.node.as_ref() else {
                            continue;
                        };
                        let value_type = infer_expr(r, value);
                        check(r, Some(value), value_type, *target, location);
                    }
                }
            }
            None => {
                for (index, (value, target)) in source_types
                    .unwrap_or_default()
                    .iter()
                    .zip(&target_types)
                    .enumerate()
                {
                    let location = select_target_location(source, index).unwrap_or_default();
                    let expr = select_value(source, index);
                    check(r, expr, value.ty.clone(), *target, location);
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

    check_generated_insert(r, &target, n, source_width);

    if let Some(conflict) = &n.on_conflict_clause {
        resolve_on_conflict_clause(r, conflict, &target);
    }

    resolve_returning_clause(r, n.returning_clause.as_ref(), Level::with_item(target))
}

/// Reports an INSERT that assigns anything but `DEFAULT` to a generated column, or to a
/// `GENERATED ALWAYS` identity column without `OVERRIDING SYSTEM VALUE` or `OVERRIDING USER
/// VALUE`. Port of the checks of [`rewriteTargetListIU`] for `CMD_INSERT`: in a VALUES list,
/// every row must have `DEFAULT` for the column, and a SELECT can't assign it at all.
///
/// Targets with subscripts or fields, like `g[1]`, are not checked. Without a column list, a
/// SELECT whose `*` expands to unknown columns isn't either.
///
/// [`rewriteTargetListIU`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/rewrite/rewriteHandler.c#L808
fn check_generated_insert(
    r: &mut Resolver,
    target: &Item,
    n: &InsertStmt,
    source_width: Option<usize>,
) {
    let overriding = matches!(
        n.r#override(),
        OverridingKind::OverridingSystemValue | OverridingKind::OverridingUserValue
    );
    if target.generated_columns.is_empty()
        && (overriding || target.identity_always_columns.is_empty())
    {
        return;
    }
    // `DEFAULT VALUES` assigns nothing.
    let Some(source) = n.select_stmt.as_deref() else {
        return;
    };
    let rows = values_rows(source);

    // The assigned columns and the location of each in the column list.
    let targets: Vec<(&str, Option<i32>)> = if n.cols.is_empty() {
        // Without a column list, the values go to the first columns of the table.
        let width = match (&rows, &source.node) {
            (Some(rows), _) => rows.first().map(|row| row.len()),
            (None, Some(NodeEnum::SelectStmt(select))) => select_width(select).or(source_width),
            _ => None,
        };
        let (Some(width), Some(columns)) = (width, target.columns.as_ref()) else {
            return;
        };
        columns
            .iter()
            .take(width)
            .map(|column| (column.as_str(), None))
            .collect()
    } else {
        n.cols
            .iter()
            .map(|column| match &column.node {
                Some(NodeEnum::ResTarget(column)) if column.indirection.is_empty() => {
                    (column.name.as_str(), Some(column.location))
                }
                // Not checked: an empty name is never a generated column.
                _ => ("", None),
            })
            .collect()
    };

    for (index, (column, column_location)) in targets.into_iter().enumerate() {
        let Some(identity) = target.default_only(column, overriding) else {
            continue;
        };
        let span = match &rows {
            Some(rows) => {
                let value = rows.iter().find_map(|row| {
                    row.get(index)
                        .filter(|value| !matches!(value.node, Some(NodeEnum::SetToDefault(_))))
                });
                match value {
                    Some(value) => column_location
                        .or_else(|| node_location(value))
                        .and_then(|location| r.name_span(location)),
                    None => continue,
                }
            }
            None => match column_location {
                Some(location) => r.name_span(location),
                None => select_target_span(r, source, index, source_width),
            },
        };
        r.report_with_span(
            FindingKind::GeneratedColumnAssignment {
                column: column.to_owned(),
                insert: true,
                identity,
            },
            span,
        );
    }
}

fn select_target_location(source: &pgls_query::Node, index: usize) -> Option<i32> {
    let NodeEnum::SelectStmt(select) = source.node.as_ref()? else {
        return None;
    };
    if let Some(row) = select.values_lists.first() {
        let NodeEnum::List(row) = row.node.as_ref()? else {
            return None;
        };
        return node_location(row.items.get(index)?);
    }
    let NodeEnum::ResTarget(target) = select.target_list.get(index)?.node.as_ref()? else {
        return None;
    };
    node_location(target.val.as_deref()?)
}

/// The span of the select list entry that produces the column at `index` of a SELECT with
/// `width` columns: the `*` or `t.*` it is part of, if any. With several `*`, it's not known which
/// one expands to the column.
fn select_target_span(
    r: &Resolver,
    source: &pgls_query::Node,
    index: usize,
    width: Option<usize>,
) -> Option<TextRange> {
    let NodeEnum::SelectStmt(select) = source.node.as_ref()? else {
        return None;
    };
    let values: Vec<&NodeEnum> = select
        .target_list
        .iter()
        .map(|target| match target.node.as_ref()? {
            NodeEnum::ResTarget(target) => target.val.as_deref()?.node.as_ref(),
            _ => None,
        })
        .collect::<Option<_>>()?;
    let mut stars = (0..values.len()).filter(|&entry| expands_to_columns(values[entry]));
    let Some(star) = stars.next() else {
        return r.name_span(select_target_location(source, index)?);
    };
    if stars.next().is_some() {
        return None;
    }
    // The `*` expands to the columns the other entries don't produce.
    let star_width = (width? + 1).checked_sub(values.len())?;
    if (star..star + star_width).contains(&index) {
        let NodeEnum::ColumnRef(column) = values[star] else {
            return None;
        };
        return r.star_span(column.location);
    }
    let entry = if index < star {
        index
    } else {
        index + 1 - star_width
    };
    let NodeEnum::ResTarget(target) = select.target_list.get(entry)?.node.as_ref()? else {
        return None;
    };
    r.name_span(node_location(target.val.as_deref()?)?)
}

/// The value of the select list entry at `index` of a plain SELECT. Postgres doesn't resolve
/// the unknown literals of the source of an INSERT, so they are coerced to the target columns
/// like the values of `INSERT ... VALUES` ([`transformInsertStmt`]). Except where DISTINCT,
/// GROUP BY or ORDER BY sort them, which makes them text ([`addTargetToSortList`]), so `None`
/// for a SELECT with any of them, and for set operations, which resolve them too.
///
/// [`transformInsertStmt`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/analyze.c#L771
/// [`addTargetToSortList`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_clause.c#L3393
fn select_value(source: &pgls_query::Node, index: usize) -> Option<&NodeEnum> {
    let NodeEnum::SelectStmt(select) = source.node.as_ref()? else {
        return None;
    };
    if select.op() != SetOperation::SetopNone
        || !select.values_lists.is_empty()
        || !select.distinct_clause.is_empty()
        || !select.group_clause.is_empty()
        || !select.sort_clause.is_empty()
    {
        return None;
    }
    let mut values = select
        .target_list
        .iter()
        .map(|target| match target.node.as_ref()? {
            NodeEnum::ResTarget(target) => target.val.as_deref()?.node.as_ref(),
            _ => None,
        });
    // A `*` before the entry expands to any number of columns.
    for _ in 0..index {
        if values.next()?.is_none_or(expands_to_columns) {
            return None;
        }
    }
    values.next()?
}

pub(super) fn node_location(node: &pgls_query::Node) -> Option<i32> {
    Some(match node.node.as_ref()? {
        NodeEnum::AConst(n) => n.location,
        NodeEnum::ColumnRef(n) => n.location,
        NodeEnum::AExpr(n) => n.location,
        NodeEnum::FuncCall(n) => n.location,
        NodeEnum::TypeCast(n) => n.location,
        NodeEnum::ParamRef(n) => n.location,
        NodeEnum::CaseExpr(n) => n.location,
        NodeEnum::CoalesceExpr(n) => n.location,
        NodeEnum::MinMaxExpr(n) => n.location,
        NodeEnum::AArrayExpr(n) => n.location,
        NodeEnum::BoolExpr(n) => n.location,
        NodeEnum::NullTest(n) => n.location,
        NodeEnum::BooleanTest(n) => n.location,
        NodeEnum::SubLink(n) => n.location,
        NodeEnum::RowExpr(n) => n.location,
        NodeEnum::SqlvalueFunction(n) => n.location,
        NodeEnum::CollateClause(n) => n.location,
        NodeEnum::AIndirection(_) => return None,
        _ => return None,
    })
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
