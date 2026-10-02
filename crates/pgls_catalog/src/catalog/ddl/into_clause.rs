use pgls_query::protobuf::IntoClause;

use crate::catalog::{Catalog, overlay::rename_columns};
use crate::lookup::{ColumnInfo, Origin, RelationInfo, RelationKind};

/// Creates the relation of `CREATE TABLE AS`, `CREATE MATERIALIZED VIEW` or `SELECT INTO`.
pub(super) fn apply_into_clause(
    c: &mut Catalog,
    n: &IntoClause,
    kind: RelationKind,
    columns: Option<Vec<ColumnInfo>>,
    if_not_exists: bool,
    search_path: &[String],
) {
    let Some(range_var) = &n.rel else {
        return;
    };
    let key = c.relation_creation_key(range_var, search_path);
    if c.skip_existing_relation(&key, if_not_exists) {
        return;
    }
    let columns = rename_columns(columns, &n.col_names);
    c.define_relation(RelationInfo {
        schema: key.0,
        name: key.1,
        kind,
        columns,
        origin: Origin::File,
    });
}
