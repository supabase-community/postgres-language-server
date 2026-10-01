use pgls_query::protobuf::CreateEnumStmt;

use crate::catalog::{Catalog, names::qualified_name};

pub(super) fn apply_create_enum_stmt(c: &mut Catalog, n: &CreateEnumStmt, search_path: &[String]) {
    if let Some(name) = qualified_name(&n.type_name) {
        let (schema, name) = c.creation_key(&name, search_path);
        c.define_type_with_metadata(schema, name, None, crate::typing::TypeKind::Enum, 'E', None);
    }
}
