use pgls_query::protobuf::CreateTableAsStmt;

use super::{Resolver, resolve_opt};

/// The query of `CREATE TABLE AS` is checked on its own.
pub(super) fn resolve_create_table_as_stmt(r: &mut Resolver, n: &CreateTableAsStmt) {
    resolve_opt(r, &n.query);
}
