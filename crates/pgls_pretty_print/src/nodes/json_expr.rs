use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};
use pgls_query::{
    NodeEnum,
    protobuf::{JsonExpr, JsonExprOp, JsonWrapper},
};

pub(super) fn emit_json_expr(e: &mut EventEmitter, n: &JsonExpr) {
    e.group_start(GroupKind::JsonExpr);

    e.token(TokenKind::IDENT(keyword_for_op(n.op()).to_string()));
    e.token(TokenKind::L_PAREN);

    let mut wrote_value = false;

    if let Some(ref formatted_expr) = n.formatted_expr {
        super::emit_node(formatted_expr, e);
        wrote_value = true;
    } else if let Some(ref xpr) = n.xpr {
        super::emit_node(xpr, e);
        wrote_value = true;
    }

    if let Some(ref path_spec) = n.path_spec {
        if wrote_value {
            e.token(TokenKind::COMMA);
            e.space();
        }
        super::emit_node(path_spec, e);
        wrote_value = true;
    }

    if !n.column_name.is_empty() {
        if wrote_value {
            e.token(TokenKind::COMMA);
            e.space();
        }
        e.token(TokenKind::IDENT(n.column_name.clone()));
        wrote_value = true;
    }

    let mut clause_has_content = wrote_value;

    if !n.passing_names.is_empty() && !n.passing_values.is_empty() {
        if clause_has_content {
            e.space();
        }
        e.token(TokenKind::IDENT("PASSING".to_string()));
        e.space();

        for (idx, (name, value)) in n
            .passing_names
            .iter()
            .zip(n.passing_values.iter())
            .enumerate()
        {
            if idx > 0 {
                e.token(TokenKind::COMMA);
                e.space();
            }

            super::emit_node(value, e);

            if let Some(ref inner) = name.node {
                match inner {
                    NodeEnum::String(s) => {
                        e.space();
                        e.token(TokenKind::AS_KW);
                        e.space();
                        super::emit_string_identifier(e, s);
                    }
                    _ => {
                        e.space();
                        e.token(TokenKind::AS_KW);
                        e.space();
                        super::emit_node_enum(inner, e);
                    }
                }
            }
        }

        clause_has_content = true;
    }

    if let Some(ref returning) = n.returning {
        super::json_value_expr::emit_json_returning_clause(e, returning, &mut clause_has_content);
    }

    if let Some(ref on_empty) = n.on_empty {
        if clause_has_content {
            e.space();
        }
        super::json_table::emit_json_behavior(e, on_empty);
        e.space();
        e.token(TokenKind::ON_KW);
        e.space();
        e.token(TokenKind::IDENT("EMPTY".to_string()));
        clause_has_content = true;
    }

    if let Some(ref on_error) = n.on_error {
        if clause_has_content {
            e.space();
        }
        super::json_table::emit_json_behavior(e, on_error);
        e.space();
        e.token(TokenKind::ON_KW);
        e.space();
        e.token(TokenKind::ERROR_KW);
        clause_has_content = true;
    }

    if let Some(wrapper_clause) = wrapper_clause(n.wrapper()) {
        if clause_has_content {
            e.space();
        }
        for (idx, token) in wrapper_clause.iter().enumerate() {
            if idx > 0 {
                e.space();
            }
            e.token(token.clone());
        }
        clause_has_content = true;
    }

    if n.omit_quotes {
        if clause_has_content {
            e.space();
        }
        e.token(TokenKind::IDENT("OMIT".to_string()));
        e.space();
        e.token(TokenKind::IDENT("QUOTES".to_string()));
        clause_has_content = true;
    }

    if n.use_json_coercion {
        if clause_has_content {
            e.space();
        }
        e.token(TokenKind::IDENT("JSON".to_string()));
        e.space();
        e.token(TokenKind::IDENT("COERCION".to_string()));
        clause_has_content = true;
    }

    if n.use_io_coercion {
        if clause_has_content {
            e.space();
        }
        e.token(TokenKind::IDENT("IO".to_string()));
        e.space();
        e.token(TokenKind::IDENT("COERCION".to_string()));
    }

    e.token(TokenKind::R_PAREN);

    if n.collation != 0 {
        e.space();
        super::emit_identifier(e, &format!("coll#{}", n.collation));
    }

    e.group_end();
}

fn keyword_for_op(op: JsonExprOp) -> &'static str {
    match op {
        JsonExprOp::JsonExistsOp => "JSON_EXISTS",
        JsonExprOp::JsonQueryOp => "JSON_QUERY",
        JsonExprOp::JsonValueOp => "JSON_VALUE",
        JsonExprOp::JsonTableOp => "JSON_TABLE",
        JsonExprOp::Undefined => "JSON_EXPR",
    }
}

fn wrapper_clause(wrapper: JsonWrapper) -> Option<Vec<TokenKind>> {
    match wrapper {
        JsonWrapper::JswNone => Some(vec![
            TokenKind::IDENT("WITHOUT".into()),
            TokenKind::IDENT("WRAPPER".into()),
        ]),
        JsonWrapper::JswConditional => Some(vec![
            TokenKind::IDENT("WITH".into()),
            TokenKind::IDENT("CONDITIONAL".into()),
            TokenKind::IDENT("WRAPPER".into()),
        ]),
        JsonWrapper::JswUnconditional => Some(vec![
            TokenKind::IDENT("WITH".into()),
            TokenKind::IDENT("UNCONDITIONAL".into()),
            TokenKind::IDENT("WRAPPER".into()),
        ]),
        JsonWrapper::JswUnspec | JsonWrapper::Undefined => None,
    }
}
