//! Select lists, RETURNING lists, and the target columns of INSERT and UPDATE.

use pgls_query::{
    Node, NodeEnum,
    protobuf::{ReturningClause, ReturningOptionKind},
};
use pgls_query_ext::utils::{star_qualifier, string_values};

use super::{Resolver, resolve_list};
use crate::resolve::{
    FindingKind, Reference, StarColumn,
    expr::{check_assignment, infer_expr},
    output::{QueryOutput, TargetOutput},
    scope::{Columns, DefaultOnly, Item, Level, StarEntry, find_item},
    utils::node_location,
};
use crate::typing::TypedColumn;

/// The output columns of a select list or RETURNING list, one entry at a time, whose `*`
/// expands to the FROM items of the innermost level. Port of the names of
/// [`transformTargetList`], with [`FigureColname`] for unnamed entries and
/// [`ExpandColumnRefStar`] and [`ExpandAllTables`] for `*`. The columns of `*` and `t.*` have the
/// types of their FROM item; the expressions are typed by [`infer_select_list`].
///
/// [`transformTargetList`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_target.c#L121
/// [`FigureColname`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_target.c#L1713
/// [`ExpandColumnRefStar`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_target.c#L1123
/// [`ExpandAllTables`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_target.c#L1296
pub(super) fn select_list_output(r: &Resolver, targets: &[Node]) -> QueryOutput {
    let level = r.levels.last();
    let mut star_types_missing = false;
    let targets = targets
        .iter()
        .map(|target| {
            let Some(NodeEnum::ResTarget(target)) = target.node.as_ref() else {
                return TargetOutput {
                    location: -1,
                    star: None,
                    columns: None,
                };
            };
            let value = target.val.as_deref().and_then(|value| value.node.as_ref());
            if let Some(NodeEnum::ColumnRef(column)) = value
                && let Some(qualifier) = star_qualifier(column).and_then(string_values)
            {
                let columns = level.and_then(|level| {
                    let (columns, types_missing) = star_output(level, &qualifier)?;
                    star_types_missing |= types_missing;
                    Some(columns)
                });
                return TargetOutput {
                    location: column.location,
                    star: Some(qualifier),
                    columns,
                };
            }
            let name = value.and_then(|_| r.target_name(target));
            TargetOutput {
                location: target.location,
                star: None,
                columns: name.map(|name| vec![TypedColumn { name, ty: None }]),
            }
        })
        .collect();
    let mut output = QueryOutput::new(targets);
    output.star_types_missing = star_types_missing;
    output
}

/// The columns that `*` (an empty `qualifier`) or `t.*` expands to in `level`, and whether the
/// types of some of them are missing.
fn star_output(level: &Level, qualifier: &[String]) -> Option<(Vec<TypedColumn>, bool)> {
    let (names, typed) = match qualifier {
        [] => (level.star_columns()?, level.star_typed_columns()),
        // PR 3: `t.*` is read three ways. Here it's an item of the innermost level only. The
        // types used to be read with `qualifier.last()` and a fallback to the whole `*`, which
        // gives the same columns whenever the names are known. `record_star_references` looks
        // the item up in every level.
        [name] => {
            let item = level.item(name)?;
            (item.columns.clone()?, item.typed_columns.clone())
        }
        // PR 3: `s.t.*` makes the columns of the entry, and so of the whole list, unknown.
        _ => return None,
    };
    let typed = typed.unwrap_or_default();
    let types_missing = typed.len() != names.len();
    let columns = names
        .into_iter()
        .enumerate()
        .map(|(i, name)| TypedColumn {
            name,
            ty: if types_missing {
                None
            } else {
                typed.get(i).and_then(|column| column.ty.clone())
            },
        })
        .collect();
    Some((columns, types_missing))
}

/// Types the expressions of a select list or RETURNING list whose output is `output`, the
/// [`select_list_output`] of `targets`. Each is typed once, after the other clauses of the
/// query, in order.
pub(super) fn infer_select_list(r: &mut Resolver, targets: &[Node], output: &mut QueryOutput) {
    // PR 3: the expressions are only typed if the names of all columns are known.
    if output.names().is_none() {
        return;
    }
    for (target, entry) in targets.iter().zip(&mut output.targets) {
        if entry.star.is_some() {
            continue;
        }
        let Some(NodeEnum::ResTarget(target)) = target.node.as_ref() else {
            continue;
        };
        let Some(value) = target.val.as_deref().and_then(|value| value.node.as_ref()) else {
            continue;
        };
        let ty = infer_expr(r, value);
        if let Some(column) = entry.columns.as_mut().and_then(|columns| columns.first_mut()) {
            column.ty = ty;
        }
    }
    if output.star_types_missing {
        output.clear_types();
    }
}

/// Records the columns that each `*` and `t.*` of a select list or RETURNING list expands to,
/// when references are collected. The FROM items of the list must be the innermost level.
pub(super) fn record_star_references(r: &mut Resolver, output: &QueryOutput) {
    if !r.collects_references() {
        return;
    }
    for target in &output.targets {
        let Some(qualifier) = &target.star else {
            continue;
        };
        let columns = match qualifier.as_slice() {
            [] => r
                .levels
                .last()
                .zip(target.columns.as_deref())
                .and_then(|(level, columns)| star_columns(level, columns)),
            // PR 3: unlike the output, `t.*` and `s.t.*` find their item in every level here,
            // and `s.t.*` copies the `(schema, name)` lookup of `column_ref.rs`.
            [name] => find_item(&r.levels, name).and_then(|item| item_columns(item, qualifier)),
            [schema, name] => r
                .levels
                .iter()
                .rev()
                .find_map(|level| {
                    level.items.iter().find(|item| {
                        item.schema.as_deref() == Some(schema.as_str())
                            && item.name.as_deref() == Some(name.as_str())
                    })
                })
                .and_then(|item| item_columns(item, qualifier)),
            _ => None,
        };
        if let Some(columns) = columns {
            r.record(Reference::Star {
                location: target.location,
                span: r.star_span(target.location),
                columns,
            });
        }
    }
}

/// The columns of `*`, like [`ExpandAllTables`], whose output is `output`, with the item of
/// each from [`Level::star`]. A column is qualified with the name of its FROM item when there are
/// several, and a column merged by `JOIN ... USING` or a `NATURAL` join is referenced by its bare
/// name. `None` if a column is not known, or if a name doesn't reference the column with
/// certainty.
///
/// [`ExpandAllTables`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_target.c#L1296
fn star_columns(level: &Level, output: &[TypedColumn]) -> Option<Vec<StarColumn>> {
    if level.items.is_empty() || level.has_opaque_items {
        return None;
    }
    let entries = level.star()?;
    if let [item] = level.items.as_slice() {
        return item_columns(item, &[]);
    }
    if entries.len() != output.len() {
        return None;
    }
    let names: Vec<&str> = output.iter().map(|column| column.name.as_str()).collect();
    let mut columns = Vec::new();
    for (entry, name) in entries.iter().zip(&names) {
        let qualifier = match entry {
            // The bare name must reference only the merged column.
            StarEntry::Merged(_) => {
                if names.iter().filter(|other| *other == name).count() != 1 {
                    return None;
                }
                Vec::new()
            }
            StarEntry::Item { item, .. } => {
                let item = level.items.get(*item)?;
                let item_name = item.name.as_ref()?;
                // The qualifier must reference only this item, and the name only this column.
                if level
                    .items
                    .iter()
                    .filter(|other| other.name.as_ref() == Some(item_name))
                    .count()
                    != 1
                    || item
                        .columns
                        .as_ref()?
                        .iter()
                        .filter(|column| column == name)
                        .count()
                        != 1
                {
                    return None;
                }
                vec![item_name.clone()]
            }
        };
        columns.push(StarColumn {
            qualifier,
            name: (*name).to_owned(),
        });
    }
    Some(columns)
}

/// The columns of `item`, qualified with `qualifier`. `None` if they are not known, or if two
/// have the same name, which then doesn't reference either.
fn item_columns(item: &Item, qualifier: &[String]) -> Option<Vec<StarColumn>> {
    let names = item.columns.as_ref()?;
    if names
        .iter()
        .enumerate()
        .any(|(i, name)| names[..i].contains(name))
    {
        return None;
    }
    Some(
        names
            .iter()
            .map(|name| StarColumn {
                qualifier: qualifier.to_vec(),
                name: name.clone(),
            })
            .collect(),
    )
}

/// Resolves a RETURNING clause of a statement on `target` against `level` and returns its
/// columns.
pub(super) fn resolve_returning_clause(
    r: &mut Resolver,
    clause: Option<&ReturningClause>,
    target: &Item,
    mut level: Level,
) -> Columns {
    let (options, list) = clause.map_or((&[][..], &[][..]), |clause| {
        (clause.options.as_slice(), clause.exprs.as_slice())
    });
    add_old_and_new(r, options, target, &mut level);
    r.enter_level(level);
    resolve_list(r, list);
    let mut output = select_list_output(r, list);
    record_star_references(r, &output);
    infer_select_list(r, list, &mut output);
    r.exit_level();
    r.output = output.columns();
    output.names()
}

/// Resolves the `SET` list of UPDATE, `ON CONFLICT DO UPDATE`, or `MERGE ... UPDATE`: each
/// target column must exist, each value must be assignable to it, and generated columns can
/// only be set to `DEFAULT` ([`check_generated_update`]). Port of
/// [`transformUpdateTargetList`].
///
/// [`transformUpdateTargetList`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/analyze.c#L2546
pub(super) fn resolve_set_clause(r: &mut Resolver, target: &Item, assignments: &[Node]) {
    check_generated_update(r, target, assignments);
    for assignment in assignments {
        if let Some(NodeEnum::ResTarget(assignment)) = &assignment.node {
            check_target_column(r, target, &assignment.name, assignment.location);
            // Assignments to subscripts or fields, like `a[1]` or `c.f`, aren't checked.
            if let (true, Some(column_type), Some(expr)) = (
                assignment.indirection.is_empty(),
                target.type_of(&assignment.name).flatten(),
                assignment.val.as_deref().and_then(|v| v.node.as_ref()),
            ) {
                let expr_type = infer_expr(r, expr);
                check_assignment(
                    r,
                    &assignment.name,
                    column_type,
                    Some(expr),
                    expr_type,
                    assignment
                        .val
                        .as_deref()
                        .and_then(node_location)
                        .unwrap_or(assignment.location),
                );
            }
        }
        super::resolve_node(r, assignment);
    }
}

/// Reports a column of the target relation of INSERT or UPDATE that doesn't exist, like
/// [`checkInsertTargets`] and [`transformUpdateTargetList`].
///
/// [`checkInsertTargets`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_target.c#L1018
/// [`transformUpdateTargetList`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/analyze.c#L2546
pub(super) fn check_target_column(r: &mut Resolver, target: &Item, column: &str, location: i32) {
    if target.has_column(column) == Some(false) {
        r.report(
            FindingKind::UnknownColumn {
                relation: target.name.clone(),
                column: column.to_owned(),
            },
            location,
        );
    }
}

/// Reports an UPDATE, `ON CONFLICT DO UPDATE` or `MERGE ... UPDATE` that sets a generated
/// column or a `GENERATED ALWAYS` identity column to anything but `DEFAULT`. Port of the checks
/// of [`rewriteTargetListIU`] for `CMD_UPDATE`, which no `OVERRIDING` clause affects.
/// [`RewriteQuery`] applies them to the UPDATE action of MERGE too.
///
/// Assignments to subscripts or fields of a column, like `g[1]`, are not checked.
///
/// [`rewriteTargetListIU`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/rewrite/rewriteHandler.c#L808
/// [`RewriteQuery`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/rewrite/rewriteHandler.c#L4162
fn check_generated_update(r: &mut Resolver, target: &Item, assignments: &[Node]) {
    if target.default_only_columns.is_empty() {
        return;
    }
    for assignment in assignments {
        let Some(NodeEnum::ResTarget(assignment)) = &assignment.node else {
            continue;
        };
        if !assignment.indirection.is_empty() {
            continue;
        }
        let Some(kind) = target.default_only(&assignment.name, false) else {
            continue;
        };
        let value = assignment
            .val
            .as_deref()
            .and_then(|value| value.node.as_ref());
        let is_default = match value {
            Some(NodeEnum::SetToDefault(_)) => Some(true),
            // `SET (a, g) = (1, DEFAULT)` and `SET (a, g) = (select ...)`.
            Some(NodeEnum::MultiAssignRef(multi)) => {
                match multi
                    .source
                    .as_deref()
                    .and_then(|source| source.node.as_ref())
                {
                    Some(NodeEnum::RowExpr(row)) => usize::try_from(multi.colno - 1)
                        .ok()
                        .and_then(|index| row.args.get(index))
                        .map(|value| matches!(value.node, Some(NodeEnum::SetToDefault(_)))),
                    Some(NodeEnum::SubLink(_)) => Some(false),
                    _ => None,
                }
            }
            Some(_) => Some(false),
            None => None,
        };
        if is_default == Some(false) {
            r.report(
                FindingKind::GeneratedColumnAssignment {
                    column: assignment.name.clone(),
                    insert: false,
                    identity: kind == DefaultOnly::IdentityAlways,
                },
                assignment.location,
            );
        }
    }
}

/// Since Postgres 18, `old` and `new` in a RETURNING list are the target row before and after
/// the change. They are only reachable by name. `RETURNING WITH (OLD AS o, NEW AS n)` renames
/// them; a name that is not given explicitly is only added if no visible item, in this or an
/// outer query, has that name. Port of
/// [`transformReturningClause`] and of [`addNSItemForReturning`]; an alias given twice, or one
/// that clashes with another item, is rejected by Postgres and not reported here.
///
/// [`transformReturningClause`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/analyze.c#L2661
/// [`addNSItemForReturning`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/analyze.c#L2621
fn add_old_and_new(r: &mut Resolver, options: &[Node], target: &Item, level: &mut Level) {
    if r.catalog.server_version_at_least(18) == Some(false) {
        return;
    }
    let explicit = |kind: ReturningOptionKind| {
        options
            .iter()
            .find_map(|option| match option.node.as_ref() {
                Some(NodeEnum::ReturningOption(option)) if option.option() == kind => Some(option),
                _ => None,
            })
    };
    let mut aliases = Vec::new();
    for (kind, default) in [
        (ReturningOptionKind::ReturningOptionOld, "old"),
        (ReturningOptionKind::ReturningOptionNew, "new"),
    ] {
        let name = match explicit(kind) {
            Some(option) => {
                // The location of an option is that of its `OLD` or `NEW`.
                r.record_alias_name(option.location, &option.value);
                option.value.as_str()
            }
            // `refnameNamespaceItem` searches the outer queries too.
            None if level.item(default).is_none() && find_item(&r.levels, default).is_none() => {
                default
            }
            None => continue,
        };
        let mut item = target.clone();
        item.name = Some(name.to_owned());
        item.schema = None;
        aliases.push(item);
    }
    level.qualified_only.extend(aliases);
}
