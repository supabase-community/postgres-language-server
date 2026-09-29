use pgls_query::{Node, protobuf::Alias};

use super::string::string_values;
use crate::resolve::scope::Columns;

/// Applies the column names of an alias (`AS t(a, b)`) to the first columns.
pub(super) fn apply_alias(columns: Columns, alias: Option<&Alias>) -> Columns {
    match alias {
        Some(alias) => rename_columns(columns, &alias.colnames),
        None => columns,
    }
}

/// Applies a list of column names (`WITH c(a, b) AS ...`) to the first columns.
pub(super) fn rename_columns(columns: Columns, names: &[Node]) -> Columns {
    if names.is_empty() {
        return columns;
    }
    let mut columns = columns?;
    let names = string_values(names)?;
    if names.len() > columns.len() {
        return None;
    }
    for (column, name) in columns.iter_mut().zip(names) {
        *column = name;
    }
    Some(columns)
}
