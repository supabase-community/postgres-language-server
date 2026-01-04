use pgls_query::NodeEnum;
use pgls_query::protobuf::ExplainStmt;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
};

pub(super) fn emit_explain_stmt(e: &mut EventEmitter, n: &ExplainStmt) {
    e.group_start(GroupKind::ExplainStmt);

    e.token(TokenKind::EXPLAIN_KW);

    // Options (ANALYZE, VERBOSE, COSTS, etc.)
    // EXPLAIN options use "name value" syntax without equals sign
    if !n.options.is_empty() {
        e.space();
        e.token(TokenKind::L_PAREN);
        for (idx, opt) in n.options.iter().enumerate() {
            if idx > 0 {
                e.token(TokenKind::COMMA);
                e.space();
            }
            emit_explain_option(e, opt);
        }
        e.token(TokenKind::R_PAREN);
    }

    // The query to explain
    if let Some(ref query) = n.query {
        e.line(LineType::SoftOrSpace);
        super::emit_node(query, e);
    }

    e.group_end();
}

fn emit_explain_option(e: &mut EventEmitter, node: &pgls_query::Node) {
    if let Some(NodeEnum::DefElem(def)) = &node.node {
        // Emit the option name in uppercase
        if !def.defname.is_empty() {
            e.token(TokenKind::IDENT(def.defname.to_uppercase()));
        }

        // Emit the option value if present (no equals sign for EXPLAIN)
        if let Some(ref arg) = def.arg {
            e.space();
            if let Some(node_enum) = &arg.node {
                match node_enum {
                    NodeEnum::Boolean(b) => {
                        // Boolean values use TRUE/FALSE keywords
                        if b.boolval {
                            e.token(TokenKind::TRUE_KW);
                        } else {
                            e.token(TokenKind::FALSE_KW);
                        }
                    }
                    NodeEnum::String(s) => {
                        // String values should be quoted
                        super::emit_string_literal(e, s);
                    }
                    _ => {
                        super::emit_node(arg, e);
                    }
                }
            }
        }
    } else {
        super::emit_node(node, e);
    }
}
