use pgls_query::protobuf::CreateTableAsStmt;

use super::{Resolver, resolve_node};
use crate::resolve::output::QueryOutput;

/// The query of `CREATE TABLE AS` is checked on its own. Returns its output.
pub(super) fn resolve_create_table_as_stmt(
    r: &mut Resolver,
    n: &CreateTableAsStmt,
) -> Option<QueryOutput> {
    resolve_node(r, n.query.as_deref()?)
}
