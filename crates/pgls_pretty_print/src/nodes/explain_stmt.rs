use pgls_query::protobuf::ExplainStmt;

use super::prelude::*;

pub(super) fn emit_explain_stmt(e: &mut EventEmitter, n: &ExplainStmt) {
    e.group_start(GroupKind::ExplainStmt);

    e.token(TokenKind::EXPLAIN_KW);

    // Options (ANALYZE, VERBOSE, COSTS, etc.)
    // EXPLAIN options use "name value" syntax without equals sign
    if !n.options.is_empty() {
        e.space();
        emit_parenthesized_list(e, &n.options, |opt, e| emit_explain_option(e, opt));
    }

    // The query to explain
    if let Some(ref query) = n.query {
        e.line(LineType::SoftOrSpace);
        emit_node(query, e);
    }

    e.group_end();
}

fn emit_explain_option(e: &mut EventEmitter, node: &pgls_query::Node) {
    if let Some(NodeEnum::DefElem(def)) = &node.node {
        if !def.defname.is_empty() {
            emit_keyword(e, &def.defname);
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
                        emit_string_literal(e, s);
                    }
                    _ => {
                        emit_node(arg, e);
                    }
                }
            }
        }
    } else {
        emit_node(node, e);
    }
}
