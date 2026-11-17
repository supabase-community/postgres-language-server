use pgls_query::protobuf::SelectStmt;
use pgls_query_ext::utils::select_into;

use super::into_clause::apply_into_clause;
use crate::catalog::Catalog;
use crate::lookup::RelationKind;
use crate::typing::TypedColumn;

/// `SELECT ... INTO` creates a table with the `output` columns of the query. In a set operation,
/// the `INTO` is on its left-most select.
pub(super) fn apply_select_stmt(
    c: &mut Catalog,
    n: &SelectStmt,
    search_path: &[String],
    output: Option<&[TypedColumn]>,
) {
    let Some(into) = select_into(n) else {
        return;
    };
    let columns = c.query_columns(output);
    apply_into_clause(c, into, RelationKind::Table, columns, false, search_path);
}
