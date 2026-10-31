use pgls_query::protobuf::{AlterOperatorStmt, DefElem, List};

use super::prelude::*;

pub(super) fn emit_alter_operator_stmt(e: &mut EventEmitter, n: &AlterOperatorStmt) {
    e.group_start();

    e.token(TokenKind::ALTER_KW);
    e.space();
    e.token(TokenKind::OPERATOR_KW);
    e.space();

    if let Some(ref oper) = n.opername {
        emit_operator_name(e, &oper.objname);

        if !oper.objargs.is_empty() || !oper.args_unspecified {
            e.space();
            emit_parenthesized_list(e, &oper.objargs, emit_node);
        }
    }

    if !n.options.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::SET_KW);
        e.space();
        emit_parenthesized_list(e, &n.options, |emitter, node| {
            if let Some(NodeEnum::DefElem(def)) = node.node.as_ref() {
                emit_operator_option(emitter, def);
            }
        });
    }

    e.group_end();
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
                e.token(TokenKind::NONE_KW);
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

fn emit_operator_option_arg(e: &mut EventEmitter, arg: &pgls_query::protobuf::Node) {
    match arg.node.as_ref() {
        Some(NodeEnum::Boolean(b)) => {
            e.token(TokenKind::IDENT(if b.boolval {
                "TRUE".to_string()
            } else {
                "FALSE".to_string()
            }));
        }
        Some(NodeEnum::List(list)) => emit_operator_list(e, list),
        Some(NodeEnum::String(s)) => emit_string_literal(e, s),
        _ => emit_node(e, arg),
    }
}

fn emit_operator_list(e: &mut EventEmitter, list: &List) {
    for (idx, item) in list.items.iter().enumerate() {
        if idx > 0 {
            e.token(TokenKind::DOT);
        }

        match item.node.as_ref() {
            Some(NodeEnum::String(s)) => e.token(TokenKind::IDENT(s.sval.clone())),
            _ => emit_node(e, item),
        }
    }
}
