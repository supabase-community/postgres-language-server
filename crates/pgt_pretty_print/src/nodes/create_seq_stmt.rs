use pgt_query::protobuf::CreateSeqStmt;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

pub(super) fn emit_create_seq_stmt(e: &mut EventEmitter, n: &CreateSeqStmt) {
    e.group_start(GroupKind::CreateSeqStmt);

    e.token(TokenKind::CREATE_KW);
    e.space();
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
        for (i, opt) in n.options.iter().enumerate() {
            if i > 0 {
                e.space();
            } else {
                e.space();
            }
            // Use specialized sequence option emission
            if let Some(pgt_query::NodeEnum::DefElem(def_elem)) = opt.node.as_ref() {
                super::emit_sequence_option(e, def_elem);
            } else {
                super::emit_node(opt, e);
            }
        }
    }

    e.token(TokenKind::SEMICOLON);
    e.group_end();
}
