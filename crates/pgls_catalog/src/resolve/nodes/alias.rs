use pgls_query::{Node, protobuf::Alias};
use pgls_query_ext::utils::string_values;

use crate::resolve::scope::{Columns, TypedColumns};

/// Applies the column names of an alias (`AS t(a, b)`) to the first columns.
pub(super) fn apply_alias(columns: Columns, alias: Option<&Alias>) -> Columns {
    match alias {
        Some(alias) => rename_columns(columns, &alias.colnames),
        None => columns,
    }
}

/// Applies the column names of an alias (`AS t(a, b)`) to the first typed columns.
pub(super) fn apply_typed_alias(columns: TypedColumns, alias: Option<&Alias>) -> TypedColumns {
    match alias {
        Some(alias) => rename_typed_columns(columns, &alias.colnames),
        None => columns,
    }
}

/// Applies a list of column names (`WITH c(a, b) AS ...`) to the first columns.
pub(super) fn rename_columns(columns: Columns, names: &[Node]) -> Columns {
    rename(columns, names, |column, name| *column = name)
}

/// Applies a list of column names (`WITH c(a, b) AS ...`) to the first typed columns.
pub(super) fn rename_typed_columns(columns: TypedColumns, names: &[Node]) -> TypedColumns {
    rename(columns, names, |column, name| column.name = name)
}

/// Renames the first columns, like [`buildRelationAliases`]. The columns are unknown if there
/// are more names than columns, which Postgres rejects.
///
/// [`buildRelationAliases`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_relation.c#L1224
fn rename<T>(
    columns: Option<Vec<T>>,
    names: &[Node],
    set_name: impl Fn(&mut T, String),
) -> Option<Vec<T>> {
    if names.is_empty() {
        return columns;
    }
    let mut columns = columns?;
    let names = string_values(names)?;
    if names.len() > columns.len() {
        return None;
    }
    for (column, name) in columns.iter_mut().zip(names) {
        set_name(column, name);
    }
    Some(columns)
}
