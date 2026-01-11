use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
    nodes::node_list::emit_comma_separated_list,
};
use pgls_query::{NodeEnum, protobuf::RangeTableFunc};

pub(super) fn emit_range_table_func(e: &mut EventEmitter, n: &RangeTableFunc) {
    e.group_start(GroupKind::RangeTableFunc);

    if n.lateral {
        e.token(TokenKind::LATERAL_KW);
        e.space();
    }

    e.token(TokenKind::XMLTABLE_KW);
    e.token(TokenKind::L_PAREN);
    e.line(LineType::Soft);
    e.indent_start();

    // XMLNAMESPACES clause - emitted before row expression
    // Namespaces are stored as ResTarget nodes with name=prefix, val=URI
    if !n.namespaces.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::XMLNAMESPACES_KW);
        e.token(TokenKind::L_PAREN);
        emit_comma_separated_list(e, &n.namespaces, |node, emitter| {
            // Each namespace is a ResTarget with val=URI, name=prefix
            // If name is empty, it's a DEFAULT namespace
            if let Some(NodeEnum::ResTarget(rt)) = node.node.as_ref() {
                if rt.name.is_empty() {
                    // DEFAULT namespace
                    emitter.token(TokenKind::DEFAULT_KW);
                    emitter.space();
                    if let Some(ref val) = rt.val {
                        super::emit_node(val, emitter);
                    }
                } else {
                    // Named namespace: 'uri' AS prefix
                    if let Some(ref val) = rt.val {
                        super::emit_node(val, emitter);
                    }
                    emitter.space();
                    emitter.token(TokenKind::AS_KW);
                    emitter.space();
                    super::emit_identifier_maybe_quoted(emitter, &rt.name);
                }
            } else {
                super::emit_node(node, emitter);
            }
        });
        e.token(TokenKind::R_PAREN);
        e.token(TokenKind::COMMA);
    }

    // Row expression (XPath for rows)
    // Wrap complex expressions in parentheses to keep them together
    if let Some(ref rowexpr) = n.rowexpr {
        e.line(LineType::SoftOrSpace);
        // Check if rowexpr is a complex expression that needs wrapping
        let needs_parens = matches!(
            rowexpr.node.as_ref(),
            Some(NodeEnum::AExpr(_)) | Some(NodeEnum::FuncCall(_)) | Some(NodeEnum::BoolExpr(_))
        );
        if needs_parens {
            e.token(TokenKind::L_PAREN);
        }
        super::emit_node(rowexpr, e);
        if needs_parens {
            e.token(TokenKind::R_PAREN);
        }
    }

    // PASSING clause (document expression)
    if let Some(ref docexpr) = n.docexpr {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::PASSING_KW);
        e.space();
        super::emit_node(docexpr, e);
    }

    // COLUMNS clause
    if !n.columns.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::COLUMNS_KW);
        e.indent_start();
        e.line(LineType::SoftOrSpace);
        emit_comma_separated_list(e, &n.columns, |node, emitter| {
            if let Some(NodeEnum::RangeTableFuncCol(col)) = node.node.as_ref() {
                super::range_table_func_col::emit_range_table_func_col(emitter, col);
            } else {
                super::emit_node(node, emitter);
            }
        });
        e.indent_end();
    }

    e.indent_end();
    e.line(LineType::Soft);
    e.token(TokenKind::R_PAREN);

    // Alias (emit_alias includes the AS keyword)
    if let Some(ref alias) = n.alias {
        e.line(LineType::SoftOrSpace);
        super::emit_alias(e, alias);
    }

    e.group_end();
}
