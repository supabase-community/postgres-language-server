use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind};
use pgt_query::protobuf::AlterSeqStmt;

pub(super) fn emit_alter_seq_stmt(e: &mut EventEmitter, n: &AlterSeqStmt) {
    e.group_start(GroupKind::AlterSeqStmt);

    e.token(TokenKind::ALTER_KW);
    e.space();
    e.token(TokenKind::IDENT("SEQUENCE".to_string()));
    e.space();

    if n.missing_ok {
        e.token(TokenKind::IF_KW);
        e.space();
        e.token(TokenKind::EXISTS_KW);
        e.space();
    }

    if let Some(ref sequence) = n.sequence {
        super::emit_range_var(e, sequence);
    }

    // Emit sequence options with proper SQL syntax (not comma-separated)
    if !n.options.is_empty() {
        for opt in &n.options {
            e.space();
            // Use specialized sequence option emission
            if let Some(pgt_query::NodeEnum::DefElem(def_elem)) = opt.node.as_ref() {
                super::emit_sequence_option(e, def_elem);
            } else {
                super::emit_node(opt, e);
            }
        }
    }

    // for_identity field indicates if this is part of ALTER TABLE ALTER COLUMN
    // In that case, the statement is embedded and might not need semicolon
    // But for now, we'll always emit it as a standalone statement
    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
