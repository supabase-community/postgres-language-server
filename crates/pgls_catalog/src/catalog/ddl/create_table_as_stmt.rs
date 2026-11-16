use pgls_query::protobuf::{CreateTableAsStmt, ObjectType};

use super::into_clause::apply_into_clause;
use crate::catalog::Catalog;
use crate::lookup::RelationKind;
use crate::typing::TypedColumn;

/// `CREATE TABLE AS` and `CREATE MATERIALIZED VIEW`. Models [`ExecCreateTableAs`].
///
/// [`ExecCreateTableAs`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/createas.c#L223
pub(super) fn apply_create_table_as_stmt(
    c: &mut Catalog,
    n: &CreateTableAsStmt,
    search_path: &[String],
    output: Option<&[TypedColumn]>,
) {
    let Some(into) = &n.into else {
        return;
    };
    let kind = if n.objtype() == ObjectType::ObjectMatview {
        RelationKind::MaterializedView
    } else {
        RelationKind::Table
    };
    let columns = c.query_columns(output);
    apply_into_clause(c, into, kind, columns, n.if_not_exists, search_path);
}
