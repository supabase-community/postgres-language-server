use pgls_query::protobuf::CopyStmt;

use super::{Resolver, range_var::lookup_relation, record_unresolved_node, resolve_node};

/// `COPY (query)` and `COPY table`. The `WHERE` clause of `COPY FROM` sees only the columns of
/// the table, so its names are not column lookups.
pub(super) fn resolve_copy_stmt(r: &mut Resolver, n: &CopyStmt) {
    if let Some(query) = n.query.as_deref() {
        resolve_node(r, query);
    } else if let Some(relation) = &n.relation {
        lookup_relation(r, relation);
    }
    if let Some(where_clause) = n.where_clause.as_deref() {
        record_unresolved_node(r, where_clause, false);
    }
}
