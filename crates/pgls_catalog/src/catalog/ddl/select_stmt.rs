use pgls_query::{Node, NodeEnum, protobuf::SelectStmt};

use super::into_clause::apply_into_clause;
use crate::catalog::Catalog;
use crate::view::RelationKind;

/// `SELECT ... INTO` creates a table.
pub(super) fn apply_select_stmt(c: &mut Catalog, n: &SelectStmt, search_path: &[String]) {
    let Some(into) = &n.into_clause else {
        return;
    };
    let mut query = n.clone();
    query.into_clause = None;
    let query = Node {
        node: Some(NodeEnum::SelectStmt(Box::new(query))),
    };
    let columns = c.query_columns(&query, search_path);
    apply_into_clause(c, into, RelationKind::Table, columns, false, search_path);
}
