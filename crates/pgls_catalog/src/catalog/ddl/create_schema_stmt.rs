use pgls_query::protobuf::{CreateSchemaStmt, RoleSpecType};

use crate::catalog::Catalog;
use crate::lookup::CatalogView;

pub(super) fn apply_create_schema_stmt(
    c: &mut Catalog,
    n: &CreateSchemaStmt,
    search_path: &[String],
) {
    let name = if !n.schemaname.is_empty() {
        n.schemaname.clone()
    } else {
        // `CREATE SCHEMA AUTHORIZATION role` names the schema after the role.
        match n.authrole.as_ref() {
            Some(role)
                if role.roletype() == RoleSpecType::RolespecCstring
                    && !role.rolename.is_empty() =>
            {
                role.rolename.clone()
            }
            _ => {
                c.tainted = true;
                return;
            }
        }
    };

    if n.if_not_exists && c.schema(&name).found().is_some() {
        return;
    }
    c.schemas.insert(name.clone(), true);

    // Schema elements are created in the new schema, which is searched first.
    let element_path: Vec<String> = std::iter::once(name)
        .chain(search_path.iter().cloned())
        .collect();
    for element in &n.schema_elts {
        if let Some(element) = &element.node {
            c.apply(element, &element_path);
        }
    }
}
