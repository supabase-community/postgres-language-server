use pgls_query::protobuf::CreateDomainStmt;

use crate::catalog::{Catalog, names::qualified_name};
use crate::lookup::CatalogView;

pub(super) fn apply_create_domain_stmt(
    c: &mut Catalog,
    n: &CreateDomainStmt,
    search_path: &[String],
) {
    if let Some(name) = qualified_name(&n.domainname) {
        let (schema, name) = c.creation_key(&name, search_path);
        let base = n
            .type_name
            .as_ref()
            .and_then(|ty| crate::normalize_type_name(c, ty, search_path).0);
        let category = base
            .as_ref()
            .and_then(|id| c.type_by_id(id).found())
            .and_then(|info| info.category)
            .unwrap_or('U');
        c.define_type_with_metadata(
            schema,
            name,
            None,
            crate::typing::TypeKind::Domain,
            category,
            base,
        );
    }
}
