use pgt_query::NodeEnum;
use pgt_query::protobuf::{AlterOperatorStmt, DefElem, List, ObjectWithArgs};

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
    nodes::node_list::emit_comma_separated_list,
};

use super::string::emit_identifier_maybe_quoted;

pub(super) fn emit_alter_operator_stmt(e: &mut EventEmitter, n: &AlterOperatorStmt) {
    e.group_start(GroupKind::AlterOperatorStmt);

    e.token(TokenKind::ALTER_KW);
    e.space();
    e.token(TokenKind::OPERATOR_KW);
    e.space();

    if let Some(ref oper) = n.opername {
        emit_operator_fqn(e, oper);

        if !oper.objargs.is_empty() || !oper.args_unspecified {
            e.space();
            e.token(TokenKind::L_PAREN);
            emit_comma_separated_list(e, &oper.objargs, super::emit_node);
            e.token(TokenKind::R_PAREN);
        }
    }

    if !n.options.is_empty() {
        e.space();
        e.token(TokenKind::SET_KW);
        e.space();
        e.token(TokenKind::L_PAREN);
        emit_comma_separated_list(e, &n.options, |node, emitter| {
            if let Some(NodeEnum::DefElem(def)) = node.node.as_ref() {
                emit_operator_option(emitter, def);
            }
        });
        e.token(TokenKind::R_PAREN);
    }

    e.token(TokenKind::SEMICOLON);
    e.group_end();
}

fn emit_operator_fqn(e: &mut EventEmitter, oper: &ObjectWithArgs) {
    for (idx, node) in oper.objname.iter().enumerate() {
        if idx > 0 {
            e.token(TokenKind::DOT);
        }

        match node.node.as_ref() {
            Some(NodeEnum::String(s)) => {
                if idx == oper.objname.len() - 1 {
                    e.token(TokenKind::IDENT(s.sval.clone()));
                } else {
                    emit_identifier_maybe_quoted(e, &s.sval);
                }
            }
            _ => super::emit_node(node, e),
        }
    }
}

fn emit_operator_option(e: &mut EventEmitter, def: &DefElem) {
    emit_identifier_maybe_quoted(e, &def.defname);

    match def.defname.to_ascii_lowercase().as_str() {
        "hashes" | "merges" => {
            if let Some(ref arg) = def.arg {
                e.space();
                e.token(TokenKind::IDENT("=".to_string()));
                e.space();
                emit_operator_option_arg(e, arg);
            }
        }
        "restrict" | "join" => {
            e.space();
            e.token(TokenKind::IDENT("=".to_string()));
            e.space();
            if let Some(ref arg) = def.arg {
                emit_operator_option_arg(e, arg);
            } else {
                e.token(TokenKind::IDENT("NONE".to_string()));
            }
        }
        _ => {
            if let Some(ref arg) = def.arg {
                e.space();
                e.token(TokenKind::IDENT("=".to_string()));
                e.space();
                emit_operator_option_arg(e, arg);
            }
        }
    }
}

fn emit_operator_option_arg(e: &mut EventEmitter, arg: &pgt_query::protobuf::Node) {
    match arg.node.as_ref() {
        Some(NodeEnum::Boolean(b)) => {
            e.token(TokenKind::IDENT(if b.boolval {
                "TRUE".to_string()
            } else {
                "FALSE".to_string()
            }));
        }
        Some(NodeEnum::List(list)) => emit_operator_list(e, list),
        Some(NodeEnum::String(s)) => super::emit_string_literal(e, s),
        _ => super::emit_node(arg, e),
    }
}

fn emit_operator_list(e: &mut EventEmitter, list: &List) {
    for (idx, item) in list.items.iter().enumerate() {
        if idx > 0 {
            e.token(TokenKind::DOT);
        }

        match item.node.as_ref() {
            Some(NodeEnum::String(s)) => e.token(TokenKind::IDENT(s.sval.clone())),
            _ => super::emit_node(item, e),
        }
    }
}
