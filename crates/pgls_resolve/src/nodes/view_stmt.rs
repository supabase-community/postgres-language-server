use pgls_query::protobuf::ViewStmt;

use super::{Resolver, resolve_node};
use crate::output::QueryOutput;

/// The query of a view is checked on its own. Returns its output.
pub(super) fn resolve_view_stmt(r: &mut Resolver, n: &ViewStmt) -> Option<QueryOutput> {
    resolve_node(r, n.query.as_deref()?)
}
