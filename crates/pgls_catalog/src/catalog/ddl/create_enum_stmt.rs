use pgls_query::protobuf::CreateEnumStmt;

use crate::catalog::{Catalog, names::qualified_name};

pub(super) fn apply_create_enum_stmt(c: &mut Catalog, n: &CreateEnumStmt, search_path: &[String]) {
    if let Some(name) = qualified_name(&n.type_name) {
        c.define_type(&name, None, search_path);
    }
}
