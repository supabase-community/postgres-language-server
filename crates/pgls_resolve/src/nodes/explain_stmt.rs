use pgls_query::protobuf::ExplainStmt;

use super::{Resolver, resolve_opt};

pub(super) fn resolve_explain_stmt(r: &mut Resolver, n: &ExplainStmt) {
    resolve_opt(r, &n.query);
}
