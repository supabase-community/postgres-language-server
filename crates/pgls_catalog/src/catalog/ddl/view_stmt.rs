use pgls_query::protobuf::ViewStmt;

use crate::catalog::{Catalog, overlay::rename_columns};
use crate::view::{Origin, RelationInfo, RelationKind};

pub(super) fn apply_view_stmt(c: &mut Catalog, n: &ViewStmt, search_path: &[String]) {
    let Some(range_var) = &n.view else {
        return;
    };
    let key = c.relation_creation_key(range_var, search_path);
    let columns = n
        .query
        .as_deref()
        .and_then(|query| c.query_columns(query, search_path));
    let columns = rename_columns(columns, &n.aliases);
    c.define_relation(RelationInfo {
        schema: key.0,
        name: key.1,
        kind: RelationKind::View,
        columns,
        origin: Origin::File,
    });
}
