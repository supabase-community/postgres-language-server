use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};
use pgt_query::protobuf::JsonFuncExpr;

pub(super) fn emit_json_func_expr(e: &mut EventEmitter, n: &JsonFuncExpr) {
    e.group_start(GroupKind::JsonFuncExpr);

    // Map JSON function operation types
    // JsonExistsOp = 0, JsonQueryOp = 1, JsonValueOp = 2, etc.
    match n.op {
        0 => {
            // JSON_EXISTS
            e.token(TokenKind::IDENT("JSON_EXISTS".to_string()));
            e.token(TokenKind::L_PAREN);

            if let Some(ref context) = n.context_item {
                if let Some(ref raw_expr) = context.raw_expr {
                    super::emit_node(raw_expr, e);
                }
            }

            e.token(TokenKind::COMMA);
            e.space();

            if let Some(ref pathspec) = n.pathspec {
                super::emit_node(pathspec, e);
            }

            e.token(TokenKind::R_PAREN);
        }
        1 => {
            // JSON_QUERY
            e.token(TokenKind::IDENT("JSON_QUERY".to_string()));
            e.token(TokenKind::L_PAREN);

            if let Some(ref context) = n.context_item {
                if let Some(ref raw_expr) = context.raw_expr {
                    super::emit_node(raw_expr, e);
                }
            }

            e.token(TokenKind::COMMA);
            e.space();

            if let Some(ref pathspec) = n.pathspec {
                super::emit_node(pathspec, e);
            }

            // TODO: Handle wrapper, quotes, on_empty, on_error

            e.token(TokenKind::R_PAREN);
        }
        2 => {
            // JSON_VALUE
            e.token(TokenKind::IDENT("JSON_VALUE".to_string()));
            e.token(TokenKind::L_PAREN);

            if let Some(ref context) = n.context_item {
                if let Some(ref raw_expr) = context.raw_expr {
                    super::emit_node(raw_expr, e);
                }
            }

            e.token(TokenKind::COMMA);
            e.space();

            if let Some(ref pathspec) = n.pathspec {
                super::emit_node(pathspec, e);
            }

            // TODO: Handle on_empty, on_error

            e.token(TokenKind::R_PAREN);
        }
        _ => {
            // Unknown JSON function - emit placeholder
            e.token(TokenKind::IDENT("JSON_FUNC".to_string()));
            e.token(TokenKind::L_PAREN);
            e.token(TokenKind::R_PAREN);
        }
    }

    e.group_end();
}
