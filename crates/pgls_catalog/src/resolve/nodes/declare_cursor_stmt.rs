use pgls_query::protobuf::DeclareCursorStmt;

use super::{Resolver, resolve_opt};

pub(super) fn resolve_declare_cursor_stmt(r: &mut Resolver, n: &DeclareCursorStmt) {
    resolve_opt(r, &n.query);
}
