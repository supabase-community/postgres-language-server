//! Select lists, RETURNING lists, and the target columns of INSERT and UPDATE.

use pgls_query::{Node, NodeEnum};

use super::{
    Resolver, resolve_list,
    string::{string_value, string_values},
};
use crate::resolve::{
    FindingKind, Reference, StarColumn,
    scope::{Columns, Item, Level, find_item},
};

/// The output columns of a select list or RETURNING list, whose `*` expands to the FROM items
/// of the innermost level. Port of the names of [`transformTargetList`], with [`FigureColname`]
/// for unnamed entries and [`ExpandColumnRefStar`] and [`ExpandAllTables`] for `*`.
///
/// [`transformTargetList`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_target.c#L121
/// [`FigureColname`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_target.c#L1713
/// [`ExpandColumnRefStar`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_target.c#L1123
/// [`ExpandAllTables`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_target.c#L1296
pub(super) fn target_list_columns(r: &Resolver, targets: &[Node]) -> Columns {
    let level = r.levels.last()?;
    let mut columns = Vec::new();
    for target in targets {
        let NodeEnum::ResTarget(target) = target.node.as_ref()? else {
            return None;
        };
        let value = target
            .val
            .as_deref()
            .and_then(|value| value.node.as_ref())?;

        if let NodeEnum::ColumnRef(column) = value
            && let Some((last, qualifier)) = column.fields.split_last()
            && matches!(last.node, Some(NodeEnum::AStar(_)))
        {
            let expanded = match qualifier {
                [] => level.star_columns()?,
                [name] => level.item(string_value(name)?)?.columns.clone()?,
                _ => return None,
            };
            columns.extend(expanded);
            continue;
        }

        if !target.name.is_empty() {
            columns.push(target.name.clone());
        } else {
            columns.push(r.column_name(value)?);
        }
    }
    Some(columns)
}

/// Records the columns that each `*` and `t.*` of a select list or RETURNING list expands to,
/// when references are collected. The FROM items of the list must be the innermost level.
pub(super) fn record_star_references(r: &mut Resolver, targets: &[Node]) {
    if !r.collects_references() {
        return;
    }
    for target in targets {
        let Some(NodeEnum::ResTarget(target)) = target.node.as_ref() else {
            continue;
        };
        let Some(NodeEnum::ColumnRef(column)) = target.val.as_deref().and_then(|v| v.node.as_ref())
        else {
            continue;
        };
        let Some((last, qualifier)) = column.fields.split_last() else {
            continue;
        };
        if !matches!(last.node, Some(NodeEnum::AStar(_))) {
            continue;
        }
        let columns = string_values(qualifier).and_then(|qualifier| match qualifier.as_slice() {
            [] => r.levels.last().and_then(star_columns),
            [name] => find_item(&r.levels, name).and_then(|item| item_columns(item, &qualifier)),
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
                .and_then(|item| item_columns(item, &qualifier)),
            _ => None,
        });
        if let Some(columns) = columns {
            r.record(Reference::Star {
                location: column.location,
                span: r.star_span(column.location),
                columns,
            });
        }
    }
}

/// The columns of `*`, like [`ExpandAllTables`]. A column is qualified with the name of its FROM
/// item when there are several. `None` if a column is not known, or if a column name doesn't
/// reference the column with certainty.
///
/// Columns merged by `JOIN ... USING` and `NATURAL` joins are not supported: they come first in
/// the output of their join, which the level doesn't record, and the bare name of a merged
/// column can be ambiguous.
///
/// [`ExpandAllTables`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_target.c#L1296
fn star_columns(level: &Level) -> Option<Vec<StarColumn>> {
    if level.items.is_empty()
        || level.has_natural_join
        || level.has_opaque_items
        || !level.merged_columns.is_empty()
    {
        return None;
    }
    if let [item] = level.items.as_slice() {
        return item_columns(item, &[]);
    }
    let mut columns = Vec::new();
    for (i, item) in level.items.iter().enumerate() {
        let name = item.name.as_ref()?;
        // The qualifier must reference only this item.
        if level.items[..i]
            .iter()
            .any(|other| other.name.as_ref() == Some(name))
        {
            return None;
        }
        columns.extend(item_columns(item, std::slice::from_ref(name))?);
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

/// Resolves a RETURNING list against `level` and returns its columns.
pub(super) fn resolve_returning_list(r: &mut Resolver, list: &[Node], mut level: Level) -> Columns {
    add_old_and_new(r, &mut level);
    r.enter_level(level);
    resolve_list(r, list);
    record_star_references(r, list);
    let columns = target_list_columns(r, list);
    r.output = columns.as_ref().map(|names| {
        let mut typed = Vec::new();
        for target in list {
            let Some(NodeEnum::ResTarget(target)) = target.node.as_ref() else {
                continue;
            };
            let Some(value) = target.val.as_deref().and_then(|value| value.node.as_ref()) else {
                continue;
            };
            if let NodeEnum::ColumnRef(column) = value
                && column
                    .fields
                    .last()
                    .is_some_and(|field| matches!(field.node, Some(NodeEnum::AStar(_))))
            {
                let qualifier = column.fields.iter().rev().nth(1).and_then(string_value);
                if let Some(item) =
                    qualifier.and_then(|name| r.levels.last().and_then(|level| level.item(name)))
                {
                    typed.extend(item.typed_columns.clone().unwrap_or_default());
                } else if let Some(level) = r.levels.last() {
                    typed.extend(level.star_typed_columns().unwrap_or_default());
                }
                continue;
            }
            typed.push(crate::typing::TypedColumn {
                name: if target.name.is_empty() {
                    r.column_name(value).unwrap_or_default()
                } else {
                    target.name.clone()
                },
                ty: super::super::expr::infer_expr(r, value),
            });
        }
        if typed.len() == names.len() {
            typed
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
    r.exit_level();
    columns
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

/// Since Postgres 18, `old` and `new` in a RETURNING list are the target row before and after
/// the change. They are only reachable by name, and only added if no visible item, in this or an
/// outer query, has that name. The first item of `level` is the target relation. Port of the
/// defaults in [`transformReturningClause`] and of [`addNSItemForReturning`]; `RETURNING WITH
/// (OLD AS ...)` needs a newer parser.
///
/// [`transformReturningClause`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/analyze.c#L2661
/// [`addNSItemForReturning`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/analyze.c#L2621
fn add_old_and_new(r: &Resolver, level: &mut Level) {
    if r.catalog
        .server_version_num()
        .is_some_and(|version| version < 180000)
    {
        return;
    }
    let Some(target) = level.items.first() else {
        return;
    };
    let mut aliases = Vec::new();
    for name in ["old", "new"] {
        // `refnameNamespaceItem` searches the outer queries too.
        if level.item(name).is_none() && find_item(&r.levels, name).is_none() {
            let mut item = target.clone();
            item.name = Some(name.to_owned());
            item.schema = None;
            aliases.push(item);
        }
    }
    level.qualified_only.extend(aliases);
}
