use pgls_query::protobuf::CreateExtensionStmt;

use crate::catalog::Catalog;

/// The objects of an installed extension are in the snapshot already. A new extension creates
/// objects we can't see.
pub(super) fn apply_create_extension_stmt(c: &mut Catalog, n: &CreateExtensionStmt) {
    let installed = c
        .base
        .as_ref()
        .is_some_and(|base| base.has_installed_extension(&n.extname));
    if !installed {
        c.tainted = true;
    }
}
