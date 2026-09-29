use pgls_query::protobuf::ViewStmt;

use super::{Resolver, resolve_opt};

/// The query of a view is checked on its own.
pub(super) fn resolve_view_stmt(r: &mut Resolver, n: &ViewStmt) {
    resolve_opt(r, &n.query);
}
