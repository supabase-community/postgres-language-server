use pgls_query::protobuf::CreateSeqStmt;

use crate::catalog::{Catalog, base::sequence_columns};
use crate::lookup::{Origin, RelationInfo, RelationKind};

/// `CREATE SEQUENCE`. Models [`DefineSequence`].
///
/// [`DefineSequence`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/sequence.c#L121
pub(super) fn apply_create_seq_stmt(c: &mut Catalog, n: &CreateSeqStmt, search_path: &[String]) {
    let Some(range_var) = &n.sequence else {
        return;
    };
    let Some(key) = c.relation_creation_key(range_var, search_path) else {
        return;
    };
    if c.skip_existing_relation(&key, n.if_not_exists) {
        return;
    }
    c.define_relation(RelationInfo {
        schema: key.0,
        name: key.1,
        kind: RelationKind::Other,
        columns: Some(sequence_columns(&rustc_hash::FxHashMap::default())),
        origin: Origin::File,
    });
}
