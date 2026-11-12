use pgls_query::protobuf::AlterSeqStmt;

use super::prelude::*;

pub(super) fn emit_alter_seq_stmt(e: &mut EventEmitter, n: &AlterSeqStmt) {
    e.group_start();

    e.token(TokenKind::ALTER_KW);
    e.space();
    e.token(TokenKind::SEQUENCE_KW);
    e.space();

    if n.missing_ok {
        emit_if_exists(e);
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
                emit_sequence_option(e, def_elem);
            } else {
                emit_node(e, opt);
            }
        }
    }

    e.group_end();
}
