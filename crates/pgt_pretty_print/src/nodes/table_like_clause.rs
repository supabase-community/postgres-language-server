use pgt_query::protobuf::TableLikeClause;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

pub(super) fn emit_table_like_clause(e: &mut EventEmitter, n: &TableLikeClause) {
    e.group_start(GroupKind::TableLikeClause);

    e.token(TokenKind::LIKE_KW);
    e.space();

    if let Some(ref relation) = n.relation {
        super::emit_range_var(e, relation);
    }

    // Options bitmap for INCLUDING/EXCLUDING clauses
    // For now, emit basic LIKE without options
    // TODO: Parse options bitmap to emit INCLUDING DEFAULTS, INCLUDING CONSTRAINTS, etc.

    e.group_end();
}
