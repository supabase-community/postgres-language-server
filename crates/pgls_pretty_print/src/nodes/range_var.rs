use pgls_query::protobuf::RangeVar;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
};

pub(super) fn emit_range_var(e: &mut EventEmitter, n: &RangeVar) {
    emit_range_var_impl(e, n, true);
}

/// Emit a RangeVar without ONLY keyword (for DDL contexts like CREATE TYPE)
pub(super) fn emit_range_var_name(e: &mut EventEmitter, n: &RangeVar) {
    emit_range_var_impl(e, n, false);
}

fn emit_range_var_impl(e: &mut EventEmitter, n: &RangeVar, allow_only: bool) {
    e.group_start(GroupKind::RangeVar);

    // ONLY is only valid in DML contexts (SELECT, UPDATE, DELETE, LOCK), not DDL
    if allow_only && !n.inh {
        e.token(TokenKind::ONLY_KW);
        e.space();
    }

    if !n.schemaname.is_empty() {
        super::string::emit_identifier_maybe_quoted(e, &n.schemaname);
        e.token(TokenKind::DOT);
    }

    super::string::emit_identifier_maybe_quoted(e, &n.relname);

    // Emit alias if present
    if let Some(ref alias) = n.alias {
        e.line(LineType::SoftOrSpace);
        super::emit_alias(e, alias);
    }

    e.group_end();
}
