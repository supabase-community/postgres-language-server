use pgls_query::protobuf::InferClause;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
};

use super::{node_list::emit_comma_separated_list, string::emit_identifier_maybe_quoted};

pub(super) fn emit_infer_clause(e: &mut EventEmitter, n: &InferClause) {
    e.group_start(GroupKind::InferClause);

    if !n.conname.is_empty() {
        e.token(TokenKind::ON_KW);
        e.space();
        e.token(TokenKind::CONSTRAINT_KW);
        e.space();
        emit_identifier_maybe_quoted(e, &n.conname);
    } else if !n.index_elems.is_empty() {
        e.token(TokenKind::L_PAREN);
        emit_comma_separated_list(e, &n.index_elems, super::emit_node);
        e.token(TokenKind::R_PAREN);
    }

    if let Some(ref where_clause) = n.where_clause {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::WHERE_KW);
        super::emit_clause_condition(e, where_clause);
    }

    e.group_end();
}
