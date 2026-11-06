use pgls_query::protobuf::AlterEnumStmt;

use crate::catalog::{Catalog, Entry, key, names::qualified_name};
use crate::lookup::{CatalogView, Origin};

/// `ALTER TYPE ... ADD VALUE` and `ALTER TYPE ... RENAME VALUE` ([`AlterEnum`]). The labels of an
/// enum are not modelled, so the enum only becomes one that the file changed, whose uses are no
/// longer checked against the database.
///
/// [`AlterEnum`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/typecmds.c#L1313
pub(super) fn apply_alter_enum_stmt(c: &mut Catalog, n: &AlterEnumStmt, search_path: &[String]) {
    let Some(name) = qualified_name(&n.type_name) else {
        return;
    };
    if let Some(mut type_info) = c.type_(name.schema(), &name.name, search_path).found() {
        type_info.origin = Origin::File;
        let key = key(&type_info.schema, &type_info.name);
        c.types.insert(key, Entry::Defined(type_info));
    }
}
