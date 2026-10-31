use pgls_query::protobuf::RangeVar;

use super::prelude::*;

/// Whether `ONLY` can precede the name. Only queries and the statements that change rows of a
/// table, SELECT, UPDATE, DELETE and LOCK, know it. DDL that names a type, CREATE TYPE for
/// instance, doesn't.
#[derive(Clone, Copy)]
pub(super) enum Only {
    Allowed,
    Omitted,
}

pub(super) fn emit_range_var(e: &mut EventEmitter, n: &RangeVar) {
    emit_range_var_as(e, n, Only::Allowed);
}

pub(super) fn emit_range_var_as(e: &mut EventEmitter, n: &RangeVar, only: Only) {
    // The parents of a RangeVar call this helper directly rather than emit_node, so this is the
    // only place that can emit the comments written next to the relation name.
    emit_with_comments_at(e, n.location, |e| emit_range_var_tokens(e, n, only));
}

fn emit_range_var_tokens(e: &mut EventEmitter, n: &RangeVar, only: Only) {
    e.group_start();

    if matches!(only, Only::Allowed) && !n.inh {
        e.token(TokenKind::ONLY_KW);
        e.space();
    }

    if !n.schemaname.is_empty() {
        emit_identifier_maybe_quoted(e, &n.schemaname);
        e.token(TokenKind::DOT);
    }

    emit_identifier_maybe_quoted(e, &n.relname);

    // Emit alias if present
    if let Some(ref alias) = n.alias {
        e.line(LineType::SoftOrSpace);
        super::emit_alias(e, alias);
    }

    e.group_end();
}
