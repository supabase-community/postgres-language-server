use pgls_query::protobuf::VacuumStmt;

use super::prelude::*;

pub(super) fn emit_vacuum_stmt(e: &mut EventEmitter, n: &VacuumStmt) {
    e.group_start(GroupKind::VacuumStmt);

    if n.is_vacuumcmd {
        e.token(TokenKind::VACUUM_KW);
    } else {
        e.token(TokenKind::ANALYZE_KW);
    }

    // Emit options if present
    if !n.options.is_empty() {
        e.space();
        emit_parenthesized_list(e, &n.options, |node, emitter| {
            if let Some(NodeEnum::DefElem(def)) = node.node.as_ref() {
                // Emit option name
                emit_keyword(emitter, &def.defname);
                // Emit option value if present
                if let Some(ref arg) = def.arg {
                    emitter.space();
                    emit_node(arg, emitter);
                }
            } else {
                emit_node(node, emitter);
            }
        });
    }

    // Relations to vacuum/analyze
    if !n.rels.is_empty() {
        e.line(LineType::SoftOrSpace);
        emit_comma_separated_list(e, &n.rels, emit_node);
    }

    e.group_end();
}
