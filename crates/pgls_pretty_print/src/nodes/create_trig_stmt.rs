use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
    nodes::node_list::{
        emit_comma_separated_list, emit_dot_separated_list, emit_space_separated_list,
    },
};
use pgls_query::protobuf::CreateTrigStmt;

pub(super) fn emit_create_trig_stmt(e: &mut EventEmitter, n: &CreateTrigStmt) {
    e.group_start(GroupKind::CreateTrigStmt);

    e.token(TokenKind::CREATE_KW);
    e.space();

    if n.replace {
        e.token(TokenKind::OR_KW);
        e.space();
        e.token(TokenKind::REPLACE_KW);
        e.space();
    }

    if n.isconstraint {
        e.token(TokenKind::CONSTRAINT_KW);
        e.space();
    }

    e.token(TokenKind::TRIGGER_KW);
    e.space();
    e.token(TokenKind::IDENT(n.trigname.clone()));

    // Timing: BEFORE (2), AFTER (4), INSTEAD OF (16)
    // After trigger name, break to new line for timing + events + ON table
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
    // Keep timing + events on same line
    e.space();
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
        e.token(TokenKind::TRUNCATE_KW);
    }

    // OF columns (for UPDATE triggers) - keep on same line as events
    if !n.columns.is_empty() {
        e.space();
        e.token(TokenKind::OF_KW);
        e.space();
        emit_comma_separated_list(e, &n.columns, super::emit_node);
    }

    // ON table - allow break before ON for long table names
    e.line(LineType::SoftOrSpace);
    e.token(TokenKind::ON_KW);
    e.space();
    if let Some(ref relation) = n.relation {
        super::emit_range_var(e, relation);
    }

    // FROM referenced_table_name (for constraint triggers)
    if let Some(ref constrrel) = n.constrrel {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::FROM_KW);
        e.space();
        super::emit_range_var(e, constrrel);
    }

    if n.deferrable {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::DEFERRABLE_KW);
    }

    if n.initdeferred {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::INITIALLY_KW);
        e.space();
        e.token(TokenKind::DEFERRED_KW);
    }

    // Referencing clause for transition tables
    if !n.transition_rels.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::REFERENCING_KW);
        e.space();
        emit_trigger_transitions(e, &n.transition_rels);
    }

    // FOR EACH ROW/STATEMENT
    e.line(LineType::SoftOrSpace);
    e.token(TokenKind::FOR_KW);
    e.space();
    e.token(TokenKind::EACH_KW);
    e.space();
    if n.row {
        e.token(TokenKind::ROW_KW);
    } else {
        e.token(TokenKind::STATEMENT_KW);
    }

    // WHEN condition
    if let Some(ref when) = n.when_clause {
        e.line(LineType::Hard);
        e.token(TokenKind::WHEN_KW);
        e.space();
        e.token(TokenKind::L_PAREN);
        super::emit_node(when, e);
        e.token(TokenKind::R_PAREN);
    }

    // EXECUTE FUNCTION
    e.line(LineType::SoftOrSpace);
    e.token(TokenKind::EXECUTE_KW);
    e.space();
    e.token(TokenKind::FUNCTION_KW);
    e.space();
    emit_dot_separated_list(e, &n.funcname);
    e.token(TokenKind::L_PAREN);
    if !n.args.is_empty() {
        // Arguments are string literals
        emit_comma_separated_list(e, &n.args, super::emit_node);
    }
    e.token(TokenKind::R_PAREN);

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}

fn emit_trigger_transitions(e: &mut EventEmitter, rels: &[pgls_query::Node]) {
    emit_space_separated_list(e, rels, |rel, e| {
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
    });
}
