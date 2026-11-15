use pgls_query::protobuf::ViewStmt;

use crate::catalog::{Catalog, overlay::rename_columns};
use crate::lookup::{Origin, RelationInfo, RelationKind};
use crate::typing::TypedColumn;

/// `CREATE VIEW`. Models [`DefineView`].
///
/// [`DefineView`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/view.c#L356
pub(super) fn apply_view_stmt(
    c: &mut Catalog,
    n: &ViewStmt,
    search_path: &[String],
    output: Option<&[TypedColumn]>,
) {
    let Some(range_var) = &n.view else {
        return;
    };
    let Some(key) = c.relation_creation_key(range_var, search_path) else {
        return;
    };
    let columns = c.query_columns(output);
    let columns = rename_columns(columns, &n.aliases);
    c.define_relation(RelationInfo {
        schema: key.0,
        name: key.1,
        kind: RelationKind::View,
        columns,
        origin: Origin::File,
    });
}
