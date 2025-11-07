use pgls_query::protobuf::TableLikeClause;

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
    // PostgreSQL CREATE_TABLE_LIKE_ constants from src/include/nodes/parsenodes.h:
    // DEFAULTS = 1 << 0, CONSTRAINTS = 1 << 1, IDENTITY = 1 << 2, GENERATED = 1 << 3
    // INDEXES = 1 << 4, STATISTICS = 1 << 5, STORAGE = 1 << 6, COMMENTS = 1 << 7, ALL = 0x7FFFFFFF

    const CREATE_TABLE_LIKE_ALL: u32 = 0x7FFFFFFF;
    const CREATE_TABLE_LIKE_COMMENTS: u32 = 1 << 7;
    const CREATE_TABLE_LIKE_CONSTRAINTS: u32 = 1 << 1;
    const CREATE_TABLE_LIKE_DEFAULTS: u32 = 1 << 0;
    const CREATE_TABLE_LIKE_GENERATED: u32 = 1 << 3;
    const CREATE_TABLE_LIKE_IDENTITY: u32 = 1 << 2;
    const CREATE_TABLE_LIKE_INDEXES: u32 = 1 << 4;
    const CREATE_TABLE_LIKE_STATISTICS: u32 = 1 << 5;
    const CREATE_TABLE_LIKE_STORAGE: u32 = 1 << 6;

    let options = n.options as u32;
    if options == CREATE_TABLE_LIKE_ALL {
        e.space();
        e.token(TokenKind::INCLUDING_KW);
        e.space();
        e.token(TokenKind::ALL_KW);
    } else if options != 0 {
        // Emit individual INCLUDING clauses
        let option_flags = [
            (CREATE_TABLE_LIKE_COMMENTS, "COMMENTS"),
            (CREATE_TABLE_LIKE_CONSTRAINTS, "CONSTRAINTS"),
            (CREATE_TABLE_LIKE_DEFAULTS, "DEFAULTS"),
            (CREATE_TABLE_LIKE_GENERATED, "GENERATED"),
            (CREATE_TABLE_LIKE_IDENTITY, "IDENTITY"),
            (CREATE_TABLE_LIKE_INDEXES, "INDEXES"),
            (CREATE_TABLE_LIKE_STATISTICS, "STATISTICS"),
            (CREATE_TABLE_LIKE_STORAGE, "STORAGE"),
        ];

        for (flag, name) in &option_flags {
            if options & flag != 0 {
                e.space();
                e.token(TokenKind::INCLUDING_KW);
                e.space();
                e.token(TokenKind::IDENT(name.to_string()));
            }
        }
    }

    e.group_end();
}
