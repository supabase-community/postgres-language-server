use pgls_query::protobuf::CreateExtensionStmt;

use crate::catalog::Catalog;

/// The objects of an installed extension are in the snapshot already. A new extension creates
/// objects we can't see. See [`CreateExtension`].
///
/// [`CreateExtension`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/extension.c#L2094
pub(super) fn apply_create_extension_stmt(c: &mut Catalog, n: &CreateExtensionStmt) {
    let installed = c
        .base
        .as_ref()
        .is_some_and(|base| base.has_installed_extension(&n.extname));
    if !installed {
        c.tainted = true;
    }
}
