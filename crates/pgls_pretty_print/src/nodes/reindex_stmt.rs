use pgls_query::{NodeEnum, protobuf::ReindexStmt};

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
    nodes::node_list::emit_comma_separated_list,
};

pub(super) fn emit_reindex_stmt(e: &mut EventEmitter, n: &ReindexStmt) {
    e.group_start(GroupKind::ReindexStmt);

    e.token(TokenKind::REINDEX_KW);

    // Handle options (CONCURRENTLY, VERBOSE, TABLESPACE, etc.)
    if !n.params.is_empty() {
        e.space();
        e.token(TokenKind::L_PAREN);
        emit_comma_separated_list(e, &n.params, |node, emitter| {
            if let Some(NodeEnum::DefElem(def)) = node.node.as_ref() {
                // Emit option name
                emitter.token(TokenKind::IDENT(def.defname.to_uppercase()));
                // Emit option value if present
                if let Some(ref arg) = def.arg {
                    emitter.space();
                    super::emit_node(arg, emitter);
                }
            } else {
                super::emit_node(node, emitter);
            }
        });
        e.token(TokenKind::R_PAREN);
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
        e.token(TokenKind::IDENT(n.name.clone()));
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
