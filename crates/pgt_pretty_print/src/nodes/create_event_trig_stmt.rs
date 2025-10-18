use pgt_query::protobuf::CreateEventTrigStmt;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

pub(super) fn emit_create_event_trig_stmt(e: &mut EventEmitter, n: &CreateEventTrigStmt) {
    e.group_start(GroupKind::CreateEventTrigStmt);

    e.token(TokenKind::CREATE_KW);
    e.space();
    e.token(TokenKind::EVENT_KW);
    e.space();
    e.token(TokenKind::TRIGGER_KW);

    // Trigger name
    if !n.trigname.is_empty() {
        e.space();
        super::emit_identifier(e, &n.trigname);
    }

    // ON event_name
    if !n.eventname.is_empty() {
        e.space();
        e.token(TokenKind::ON_KW);
        e.space();
        e.token(TokenKind::IDENT(n.eventname.clone()));
    }

    // WHEN clause (optional)
    // Format: WHEN TAG IN ('value1', 'value2') AND TAG IN ('value3')
    if !n.whenclause.is_empty() {
        e.space();
        e.token(TokenKind::WHEN_KW);
        e.space();
        for (i, when) in n.whenclause.iter().enumerate() {
            if i > 0 {
                e.space();
                e.token(TokenKind::AND_KW);
                e.space();
            }
            // Each when clause is a DefElem with defname=tag and arg=List of values
            if let Some(pgt_query::NodeEnum::DefElem(def_elem)) = when.node.as_ref() {
                // Emit TAG name (uppercased)
                e.token(TokenKind::IDENT(def_elem.defname.to_uppercase()));
                e.space();
                e.token(TokenKind::IN_KW);
                e.space();
                e.token(TokenKind::L_PAREN);
                // Emit list of values
                if let Some(arg) = &def_elem.arg {
                    if let Some(pgt_query::NodeEnum::List(list)) = arg.node.as_ref() {
                        for (j, item) in list.items.iter().enumerate() {
                            if j > 0 {
                                e.token(TokenKind::COMMA);
                                e.space();
                            }
                            if let Some(pgt_query::NodeEnum::String(s)) = item.node.as_ref() {
                                super::emit_string_literal(e, s);
                            }
                        }
                    }
                }
                e.token(TokenKind::R_PAREN);
            }
        }
    }

    // EXECUTE FUNCTION function_name()
    if !n.funcname.is_empty() {
        e.space();
        e.token(TokenKind::EXECUTE_KW);
        e.space();
        e.token(TokenKind::FUNCTION_KW);
        e.space();
        super::node_list::emit_dot_separated_list(e, &n.funcname);
        e.token(TokenKind::L_PAREN);
        e.token(TokenKind::R_PAREN);
    }

    e.token(TokenKind::SEMICOLON);
    e.group_end();
}
