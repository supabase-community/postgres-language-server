use pgls_query::protobuf::CopyStmt;

use super::{Resolver, range_var::lookup_relation, resolve_node};

/// `COPY (query)` and `COPY table`.
pub(super) fn resolve_copy_stmt(r: &mut Resolver, n: &CopyStmt) {
    if let Some(query) = n.query.as_deref() {
        resolve_node(r, query);
    } else if let Some(relation) = &n.relation {
        lookup_relation(r, relation);
    }
}
