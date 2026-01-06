use pgls_query::protobuf::CreateSeqStmt;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
};

use super::node_list::emit_space_separated_list;

pub(super) fn emit_create_seq_stmt(e: &mut EventEmitter, n: &CreateSeqStmt) {
    e.group_start(GroupKind::CreateSeqStmt);

    e.token(TokenKind::CREATE_KW);
    e.space();

    // Handle TEMPORARY/TEMP sequences
    if let Some(ref seq) = n.sequence {
        if seq.relpersistence == "t" {
            e.token(TokenKind::TEMPORARY_KW);
            e.space();
        } else if seq.relpersistence == "u" {
            // UNLOGGED
            e.token(TokenKind::UNLOGGED_KW);
            e.space();
        }
    }

    e.token(TokenKind::SEQUENCE_KW);

    if n.if_not_exists {
        e.space();
        e.token(TokenKind::IF_KW);
        e.space();
        e.token(TokenKind::NOT_KW);
        e.space();
        e.token(TokenKind::EXISTS_KW);
    }

    if let Some(ref sequence) = n.sequence {
        e.space();
        super::emit_range_var(e, sequence);
    }

    // Emit sequence options (AS type, INCREMENT BY, MINVALUE, MAXVALUE, START WITH, CACHE, CYCLE, etc.)
    if !n.options.is_empty() {
        e.line(LineType::SoftOrSpace);
        emit_space_separated_list(e, &n.options, |opt, e| {
            // Use specialized sequence option emission
            if let Some(pgls_query::NodeEnum::DefElem(def_elem)) = opt.node.as_ref() {
                super::emit_sequence_option(e, def_elem);
            } else {
                super::emit_node(opt, e);
            }
        });
    }

    e.token(TokenKind::SEMICOLON);
    e.group_end();
}
