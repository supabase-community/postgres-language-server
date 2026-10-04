//! Select lists, RETURNING lists, and the target columns of INSERT and UPDATE.

use pgls_query::{Node, NodeEnum};

use super::{Resolver, resolve_list, string::string_value};
use crate::resolve::column_name::figure_column_name;
use crate::resolve::{
    FindingKind,
    scope::{Columns, Item, Level},
};

/// The output columns of a select list or RETURNING list, whose `*` expands to the FROM items
/// of the innermost level.
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
            columns.push(figure_column_name(value)?);
        }
    }
    Some(columns)
}

/// Resolves a RETURNING list against `level` and returns its columns.
pub(super) fn resolve_returning_list(r: &mut Resolver, list: &[Node], level: Level) -> Columns {
    r.enter_level(level);
    resolve_list(r, list);
    let columns = target_list_columns(r, list);
    r.exit_level();
    columns
}

/// Reports a column of the target relation of INSERT or UPDATE that doesn't exist.
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
