use pgls_query::protobuf::CreateEventTrigStmt;

use super::prelude::*;

pub(super) fn emit_create_event_trig_stmt(e: &mut EventEmitter, n: &CreateEventTrigStmt) {
    e.group_start();

    e.token(TokenKind::CREATE_KW);
    e.space();
    e.token(TokenKind::EVENT_KW);
    e.space();
    e.token(TokenKind::TRIGGER_KW);

    // Trigger name
    if !n.trigname.is_empty() {
        e.space();
        emit_identifier_maybe_quoted(e, &n.trigname);
    }

    // ON event_name
    if !n.eventname.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::ON_KW);
        e.space();
        e.token(TokenKind::IDENT(n.eventname.clone()));
    }

    // WHEN clause (optional)
    // Format: WHEN TAG IN ('value1', 'value2') AND TAG IN ('value3')
    if !n.whenclause.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::WHEN_KW);
        e.space();
        for (i, when) in n.whenclause.iter().enumerate() {
            if i > 0 {
                e.space();
                e.token(TokenKind::AND_KW);
                e.space();
            }
            // Each when clause is a DefElem with defname=tag and arg=List of values
            if let Some(pgls_query::NodeEnum::DefElem(def_elem)) = when.node.as_ref() {
                // Emit TAG name
                emit_keyword(e, &def_elem.defname);
                e.space();
                e.token(TokenKind::IN_KW);
                e.space();
                // Emit list of values, filling indented lines when they don't fit
                if let Some(arg) = &def_elem.arg
                    && let Some(pgls_query::NodeEnum::List(list)) = arg.node.as_ref()
                {
                    e.group_start();
                    open_call_arguments(e);
                    emit_fill_comma_separated_list(e, &list.items, |e, item| {
                        if let Some(pgls_query::NodeEnum::String(s)) = item.node.as_ref() {
                            emit_string_literal(e, s);
                        }
                    });
                    close_call_arguments(e);
                    e.group_end();
                } else {
                    e.token(TokenKind::L_PAREN);
                    e.token(TokenKind::R_PAREN);
                }
            }
        }
    }

    // EXECUTE FUNCTION function_name()
    if !n.funcname.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::EXECUTE_KW);
        e.space();
        e.token(TokenKind::FUNCTION_KW);
        e.space();
        emit_dot_separated_list(e, &n.funcname);
        e.token(TokenKind::L_PAREN);
        e.token(TokenKind::R_PAREN);
    }

    e.group_end();
}
