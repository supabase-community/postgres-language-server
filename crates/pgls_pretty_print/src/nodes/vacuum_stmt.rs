use pgls_query::{NodeEnum, protobuf::VacuumStmt};

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
};

use super::node_list::emit_comma_separated_list;

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
        e.token(TokenKind::L_PAREN);
        emit_comma_separated_list(e, &n.options, |node, emitter| {
            if let Some(NodeEnum::DefElem(def)) = node.node.as_ref() {
                // Emit option name
                emitter.token(TokenKind::IDENT(def.defname.to_uppercase()));
                // Emit option value if present
                if let Some(ref arg) = def.arg {
                    emitter.space();
                    super::emit_node(arg, emitter);
                }
            } else {
                super::emit_node(node, emitter);
            }
        });
        e.token(TokenKind::R_PAREN);
    }

    // Relations to vacuum/analyze
    if !n.rels.is_empty() {
        e.line(LineType::SoftOrSpace);
        emit_comma_separated_list(e, &n.rels, super::emit_node);
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
