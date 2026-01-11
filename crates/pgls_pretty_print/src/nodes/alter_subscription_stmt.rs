use super::{
    node_list::emit_comma_separated_list,
    string::{emit_identifier_maybe_quoted, emit_keyword, emit_single_quoted_str},
};
use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
};
use pgls_query::protobuf::AlterSubscriptionStmt;

pub(super) fn emit_alter_subscription_stmt(e: &mut EventEmitter, n: &AlterSubscriptionStmt) {
    e.group_start(GroupKind::AlterSubscriptionStmt);

    e.token(TokenKind::ALTER_KW);
    e.space();
    emit_keyword(e, "SUBSCRIPTION");
    e.space();
    emit_identifier_maybe_quoted(e, &n.subname);

    // Kind enum: 0=Undefined, 1=OPTIONS, 2=CONNECTION, 3=SET_PUBLICATION, 4=ADD_PUBLICATION, 5=DROP_PUBLICATION, 6=REFRESH, 7=ENABLED, 8=SKIP
    match n.kind {
        1 => {
            // OPTIONS - uses SET (options), handled below
            e.line(LineType::SoftOrSpace);
            e.token(TokenKind::SET_KW);
        }
        2 => {
            e.line(LineType::SoftOrSpace);
            emit_keyword(e, "CONNECTION");
            e.space();
            emit_single_quoted_str(e, &n.conninfo);
        }
        3 => {
            e.line(LineType::SoftOrSpace);
            e.token(TokenKind::SET_KW);
            e.space();
            emit_keyword(e, "PUBLICATION");
            e.line(LineType::SoftOrSpace);
            emit_comma_separated_list(e, &n.publication, super::emit_node);
        }
        4 => {
            e.line(LineType::SoftOrSpace);
            emit_keyword(e, "ADD");
            e.space();
            emit_keyword(e, "PUBLICATION");
            e.line(LineType::SoftOrSpace);
            emit_comma_separated_list(e, &n.publication, super::emit_node);
        }
        5 => {
            e.line(LineType::SoftOrSpace);
            e.token(TokenKind::DROP_KW);
            e.space();
            emit_keyword(e, "PUBLICATION");
            e.line(LineType::SoftOrSpace);
            emit_comma_separated_list(e, &n.publication, super::emit_node);
        }
        6 => {
            e.space();
            emit_keyword(e, "REFRESH");
            e.space();
            emit_keyword(e, "PUBLICATION");
        }
        7 => {
            // ENABLE or DISABLE - determined by the enabled option value
            let is_enabled = n.options.iter().any(|opt| {
                if let Some(pgls_query::NodeEnum::DefElem(def)) = &opt.node {
                    if def.defname == "enabled" {
                        if let Some(arg) = &def.arg {
                            if let Some(pgls_query::NodeEnum::Boolean(b)) = &arg.node {
                                return b.boolval;
                            }
                        }
                    }
                }
                false
            });
            e.space();
            if is_enabled {
                emit_keyword(e, "ENABLE");
            } else {
                emit_keyword(e, "DISABLE");
            }
        }
        8 => {
            e.space();
            emit_keyword(e, "SKIP");
        }
        _ => {}
    }

    // Options - only emit for kinds that support options
    // ENABLE (7) stores internal state in options but doesn't emit it in SQL
    if !n.options.is_empty() && n.kind != 7 {
        // For kind 1 (SET) and kind 8 (SKIP), we already emitted the keyword, just need the parens
        // For other kinds, we need WITH
        if n.kind == 1 || n.kind == 8 {
            e.line(LineType::SoftOrSpace);
        } else {
            e.line(LineType::SoftOrSpace);
            e.token(TokenKind::WITH_KW);
            e.space();
        }
        e.token(TokenKind::L_PAREN);
        emit_comma_separated_list(e, &n.options, super::emit_node);
        e.token(TokenKind::R_PAREN);
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
