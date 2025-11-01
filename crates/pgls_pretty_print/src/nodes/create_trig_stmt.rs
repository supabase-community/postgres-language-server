use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
    nodes::node_list::emit_dot_separated_list,
};
use pgls_query::protobuf::CreateTrigStmt;

pub(super) fn emit_create_trig_stmt(e: &mut EventEmitter, n: &CreateTrigStmt) {
    e.group_start(GroupKind::CreateTrigStmt);

    e.token(TokenKind::CREATE_KW);
    e.space();

    if n.replace {
        e.token(TokenKind::OR_KW);
        e.space();
        e.token(TokenKind::IDENT("REPLACE".to_string()));
        e.space();
    }

    if n.isconstraint {
        e.token(TokenKind::IDENT("CONSTRAINT".to_string()));
        e.space();
    }

    e.token(TokenKind::IDENT("TRIGGER".to_string()));
    e.space();
    e.token(TokenKind::IDENT(n.trigname.clone()));

    // Timing: BEFORE (2), AFTER (4), INSTEAD OF (16)
    e.line(LineType::SoftOrSpace);
    let timing = n.timing;
    if timing & (1 << 6) != 0 {
        e.token(TokenKind::INSTEAD_KW);
        e.space();
        e.token(TokenKind::OF_KW);
    } else if timing & (1 << 1) != 0 {
        e.token(TokenKind::BEFORE_KW);
    } else {
        e.token(TokenKind::AFTER_KW);
    }

    // Events: INSERT (4), DELETE (8), UPDATE (16), TRUNCATE (32)
    e.line(LineType::SoftOrSpace);
    let mut first_event = true;
    if n.events & 4 != 0 {
        e.token(TokenKind::INSERT_KW);
        first_event = false;
    }
    if n.events & 8 != 0 {
        if !first_event {
            e.space();
            e.token(TokenKind::OR_KW);
            e.space();
        }
        e.token(TokenKind::DELETE_KW);
        first_event = false;
    }
    if n.events & 16 != 0 {
        if !first_event {
            e.space();
            e.token(TokenKind::OR_KW);
            e.space();
        }
        e.token(TokenKind::UPDATE_KW);
        first_event = false;
    }
    if n.events & 32 != 0 {
        if !first_event {
            e.space();
            e.token(TokenKind::OR_KW);
            e.space();
        }
        e.token(TokenKind::IDENT("TRUNCATE".to_string()));
    }

    // OF columns (for UPDATE triggers)
    if !n.columns.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::OF_KW);
        e.space();
        emit_dot_separated_list(e, &n.columns);
    }

    e.line(LineType::SoftOrSpace);
    e.token(TokenKind::ON_KW);
    e.space();
    if let Some(ref relation) = n.relation {
        super::emit_range_var(e, relation);
    }

    if n.deferrable {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::IDENT("DEFERRABLE".to_string()));
    }

    if n.initdeferred {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::IDENT("INITIALLY".to_string()));
        e.space();
        e.token(TokenKind::IDENT("DEFERRED".to_string()));
    }

    // Referencing clause for transition tables
    if !n.transition_rels.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::IDENT("REFERENCING".to_string()));
        e.space();
        emit_trigger_transitions(e, &n.transition_rels);
    }

    // FOR EACH ROW/STATEMENT
    e.line(LineType::SoftOrSpace);
    e.token(TokenKind::FOR_KW);
    e.space();
    e.token(TokenKind::IDENT("EACH".to_string()));
    e.space();
    if n.row {
        e.token(TokenKind::IDENT("ROW".to_string()));
    } else {
        e.token(TokenKind::IDENT("STATEMENT".to_string()));
    }

    // WHEN condition
    if let Some(ref when) = n.when_clause {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::WHEN_KW);
        e.space();
        e.token(TokenKind::L_PAREN);
        super::emit_node(when, e);
        e.token(TokenKind::R_PAREN);
    }

    // EXECUTE FUNCTION
    e.line(LineType::SoftOrSpace);
    e.token(TokenKind::IDENT("EXECUTE".to_string()));
    e.space();
    e.token(TokenKind::IDENT("FUNCTION".to_string()));
    e.space();
    emit_dot_separated_list(e, &n.funcname);
    e.token(TokenKind::L_PAREN);
    if !n.args.is_empty() {
        // Arguments are string literals
        for (i, arg) in n.args.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::COMMA);
                e.space();
            }
            super::emit_node(arg, e);
        }
    }
    e.token(TokenKind::R_PAREN);

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}

fn emit_trigger_transitions(e: &mut EventEmitter, rels: &[pgls_query::Node]) {
    for (idx, rel) in rels.iter().enumerate() {
        if idx > 0 {
            e.space();
        }

        let transition = assert_node_variant!(TriggerTransition, rel);

        if transition.is_new {
            e.token(TokenKind::NEW_KW);
        } else {
            e.token(TokenKind::OLD_KW);
        }

        e.space();

        if transition.is_table {
            e.token(TokenKind::TABLE_KW);
        } else {
            e.token(TokenKind::ROW_KW);
        }

        e.space();
        e.token(TokenKind::AS_KW);
        e.space();
        e.token(TokenKind::IDENT(transition.name.clone()));
    }
}
