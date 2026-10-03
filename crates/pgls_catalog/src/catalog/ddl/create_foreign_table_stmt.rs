use pgls_query::protobuf::CreateForeignTableStmt;

use super::create_stmt::create_table;
use crate::catalog::Catalog;
use crate::lookup::RelationKind;

/// `CREATE FOREIGN TABLE`, which [`DefineRelation`] creates like a table.
///
/// [`DefineRelation`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/tablecmds.c#L765
pub(super) fn apply_create_foreign_table_stmt(
    c: &mut Catalog,
    n: &CreateForeignTableStmt,
    search_path: &[String],
) {
    if let Some(base) = &n.base_stmt {
        create_table(c, base, RelationKind::ForeignTable, search_path);
    }
}
