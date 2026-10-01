use pgls_query::protobuf::{CreateTableAsStmt, ObjectType};

use super::into_clause::apply_into_clause;
use crate::catalog::Catalog;
use crate::lookup::RelationKind;

/// `CREATE TABLE AS` and `CREATE MATERIALIZED VIEW`.
pub(super) fn apply_create_table_as_stmt(
    c: &mut Catalog,
    n: &CreateTableAsStmt,
    search_path: &[String],
) {
    let Some(into) = &n.into else {
        return;
    };
    let kind = if n.objtype() == ObjectType::ObjectMatview {
        RelationKind::MaterializedView
    } else {
        RelationKind::Table
    };
    let columns = n
        .query
        .as_deref()
        .and_then(|query| c.query_columns(query, search_path));
    apply_into_clause(c, into, kind, columns, n.if_not_exists, search_path);
}
