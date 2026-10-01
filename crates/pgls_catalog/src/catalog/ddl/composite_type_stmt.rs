use pgls_query::{NodeEnum, protobuf::CompositeTypeStmt};

use crate::catalog::{Catalog, Entry, names::range_var_name, overlay::column_info};

/// `CREATE TYPE name AS (...)`.
pub(super) fn apply_composite_type_stmt(
    c: &mut Catalog,
    n: &CompositeTypeStmt,
    search_path: &[String],
) {
    let Some(range_var) = &n.typevar else {
        return;
    };
    let attributes = n
        .coldeflist
        .iter()
        .filter_map(|node| match &node.node {
            Some(NodeEnum::ColumnDef(column)) => Some(column_info(c, column, search_path)),
            _ => None,
        })
        .collect();
    let name = range_var_name(range_var);
    let key = c.creation_key(&name, search_path);
    c.define_type(&name, Some(attributes), search_path);
    // Composite types are relations, too, but they can't be queried.
    c.relations.insert(key, Entry::Unknown);
}
