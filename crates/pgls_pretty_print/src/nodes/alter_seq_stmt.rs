use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind, LineType};
use pgls_query::protobuf::AlterSeqStmt;

pub(super) fn emit_alter_seq_stmt(e: &mut EventEmitter, n: &AlterSeqStmt) {
    e.group_start(GroupKind::AlterSeqStmt);

    e.token(TokenKind::ALTER_KW);
    e.space();
    e.token(TokenKind::SEQUENCE_KW);
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
            e.line(LineType::SoftOrSpace);
            // Use specialized sequence option emission
            if let Some(pgls_query::NodeEnum::DefElem(def_elem)) = opt.node.as_ref() {
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
