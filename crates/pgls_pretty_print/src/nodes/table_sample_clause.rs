use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
    nodes::node_list::emit_comma_separated_list,
};
use pgls_query::protobuf::TableSampleClause;

pub(super) fn emit_table_sample_clause(e: &mut EventEmitter, clause: &TableSampleClause) {
    e.group_start(GroupKind::TableSampleClause);

    e.token(TokenKind::IDENT("TABLESAMPLE".to_string()));
    e.space();

    if clause.tsmhandler != 0 {
        super::emit_identifier(e, &format!("handler#{}", clause.tsmhandler));
    } else {
        super::emit_identifier_maybe_quoted(e, "handler");
    }

    if !clause.args.is_empty() {
        e.token(TokenKind::L_PAREN);
        emit_comma_separated_list(e, &clause.args, super::emit_node);
        e.token(TokenKind::R_PAREN);
    }

    if let Some(repeatable) = clause.repeatable.as_ref() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::IDENT("REPEATABLE".to_string()));
        e.space();
        e.token(TokenKind::L_PAREN);
        super::emit_node(repeatable, e);
        e.token(TokenKind::R_PAREN);
    }

    e.group_end();
}
