use pgls_query::protobuf::CreateSeqStmt;

use crate::catalog::{Catalog, base::sequence_columns};
use crate::view::{Origin, RelationInfo, RelationKind};

pub(super) fn apply_create_seq_stmt(c: &mut Catalog, n: &CreateSeqStmt, search_path: &[String]) {
    let Some(range_var) = &n.sequence else {
        return;
    };
    let key = c.relation_creation_key(range_var, search_path);
    if c.skip_existing_relation(&key, n.if_not_exists) {
        return;
    }
    c.define_relation(RelationInfo {
        schema: key.0,
        name: key.1,
        kind: RelationKind::Other,
        columns: Some(sequence_columns()),
        origin: Origin::File,
    });
}
