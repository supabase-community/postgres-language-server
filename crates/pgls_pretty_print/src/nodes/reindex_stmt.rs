use pgls_query::{NodeEnum, protobuf::ReindexStmt};

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
};

use super::node_list::emit_parenthesized_list;

pub(super) fn emit_reindex_stmt(e: &mut EventEmitter, n: &ReindexStmt) {
    e.group_start(GroupKind::ReindexStmt);

    e.token(TokenKind::REINDEX_KW);

    // Handle options (CONCURRENTLY, VERBOSE, TABLESPACE, etc.)
    if !n.params.is_empty() {
        e.space();
        emit_parenthesized_list(e, &n.params, |node, emitter| {
            if let Some(NodeEnum::DefElem(def)) = node.node.as_ref() {
                // Emit option name
                super::emit_word(emitter, &def.defname);
                // Emit option value if present
                if let Some(ref arg) = def.arg {
                    emitter.space();
                    super::emit_node(arg, emitter);
                }
            } else {
                super::emit_node(node, emitter);
            }
        });
    }

    e.line(LineType::SoftOrSpace);

    // ReindexObjectType enum:
    // 0: Undefined
    // 1: REINDEX_OBJECT_INDEX
    // 2: REINDEX_OBJECT_TABLE
    // 3: REINDEX_OBJECT_SCHEMA
    // 4: REINDEX_OBJECT_SYSTEM
    // 5: REINDEX_OBJECT_DATABASE
    match n.kind {
        1 => e.token(TokenKind::INDEX_KW),
        2 => e.token(TokenKind::TABLE_KW),
        3 => e.token(TokenKind::SCHEMA_KW),
        4 => e.token(TokenKind::SYSTEM_KW),
        5 => e.token(TokenKind::DATABASE_KW),
        _ => e.token(TokenKind::TABLE_KW), // default
    }

    e.space();

    // Either relation or name is used
    if let Some(ref relation) = n.relation {
        super::emit_range_var(e, relation);
    } else if !n.name.is_empty() {
        super::emit_identifier_maybe_quoted(e, &n.name);
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
