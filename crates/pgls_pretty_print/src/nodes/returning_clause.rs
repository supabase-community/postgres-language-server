use pgls_query::{
    NodeEnum,
    protobuf::{ReturningClause, ReturningOption, ReturningOptionKind},
};

use crate::{TokenKind, emitter::EventEmitter};

/// Emits `RETURNING [WITH (OLD AS o, NEW AS n)] expr, ...` of an INSERT, UPDATE or DELETE.
///
/// It opens no group, so the list breaks with the statement around it.
pub(super) fn emit_returning_clause(e: &mut EventEmitter, n: &ReturningClause) {
    emit_returning_keyword(e, n);
    super::node_list::emit_comma_separated_list(e, &n.exprs, super::emit_node);
}

/// Emits `RETURNING [WITH (OLD AS o, NEW AS n)] `, for a caller that lays out the list itself.
pub(super) fn emit_returning_keyword(e: &mut EventEmitter, n: &ReturningClause) {
    e.token(TokenKind::RETURNING_KW);
    e.space();
    if !n.options.is_empty() {
        e.token(TokenKind::WITH_KW);
        e.space();
        e.token(TokenKind::L_PAREN);
        for (index, option) in n.options.iter().enumerate() {
            if index > 0 {
                e.token(TokenKind::COMMA);
                e.space();
            }
            if let Some(NodeEnum::ReturningOption(option)) = &option.node {
                emit_returning_option(e, option);
            }
        }
        e.token(TokenKind::R_PAREN);
        e.space();
    }
}

/// Emits `OLD AS alias` or `NEW AS alias` of `RETURNING WITH (...)`.
pub(super) fn emit_returning_option(e: &mut EventEmitter, n: &ReturningOption) {
    match n.option() {
        ReturningOptionKind::ReturningOptionOld => e.token(TokenKind::OLD_KW),
        ReturningOptionKind::ReturningOptionNew => e.token(TokenKind::NEW_KW),
        ReturningOptionKind::Undefined => return,
    }
    e.space();
    e.token(TokenKind::AS_KW);
    e.space();
    super::emit_identifier_maybe_quoted(e, &n.value);
}
