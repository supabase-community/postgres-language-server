use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
    nodes::node_list::emit_parenthesized_list,
};
use pgls_query::protobuf::TableSampleClause;

pub(super) fn emit_table_sample_clause(e: &mut EventEmitter, clause: &TableSampleClause) {
    e.group_start(GroupKind::TableSampleClause);

    e.token(TokenKind::TABLESAMPLE_KW);
    e.space();

    if clause.tsmhandler != 0 {
        super::emit_identifier(e, &format!("handler#{}", clause.tsmhandler));
    } else {
        super::emit_identifier_maybe_quoted(e, "handler");
    }

    if !clause.args.is_empty() {
        emit_parenthesized_list(e, &clause.args, super::emit_node);
    }

    if let Some(repeatable) = clause.repeatable.as_ref() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::REPEATABLE_KW);
        e.space();
        e.token(TokenKind::L_PAREN);
        super::emit_node(repeatable, e);
        e.token(TokenKind::R_PAREN);
    }

    e.group_end();
}
