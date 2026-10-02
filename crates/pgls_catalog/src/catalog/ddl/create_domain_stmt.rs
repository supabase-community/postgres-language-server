use pgls_query::protobuf::CreateDomainStmt;

use crate::catalog::{Catalog, names::qualified_name};

pub(super) fn apply_create_domain_stmt(
    c: &mut Catalog,
    n: &CreateDomainStmt,
    search_path: &[String],
) {
    if let Some(name) = qualified_name(&n.domainname) {
        c.define_type(&name, None, search_path);
    }
}
