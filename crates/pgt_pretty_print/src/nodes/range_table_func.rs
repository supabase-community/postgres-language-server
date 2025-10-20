use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
    nodes::node_list::emit_comma_separated_list,
};
use pgt_query::{NodeEnum, protobuf::RangeTableFunc};

pub(super) fn emit_range_table_func(e: &mut EventEmitter, n: &RangeTableFunc) {
    e.group_start(GroupKind::RangeTableFunc);

    if n.lateral {
        e.token(TokenKind::IDENT("LATERAL".to_string()));
        e.space();
    }

    e.token(TokenKind::IDENT("XMLTABLE".to_string()));
    e.token(TokenKind::L_PAREN);

    // Row expression (XPath for rows)
    if let Some(ref rowexpr) = n.rowexpr {
        super::emit_node(rowexpr, e);
    }

    // PASSING clause (document expression)
    if let Some(ref docexpr) = n.docexpr {
        e.space();
        e.token(TokenKind::IDENT("PASSING".to_string()));
        e.space();
        super::emit_node(docexpr, e);
    }

    // COLUMNS clause
    if !n.columns.is_empty() {
        e.space();
        e.token(TokenKind::IDENT("COLUMNS".to_string()));
        e.space();
        emit_comma_separated_list(e, &n.columns, |node, e| {
            if let Some(NodeEnum::RangeTableFuncCol(col)) = &node.node {
                e.token(TokenKind::IDENT(col.colname.clone()));

                if col.for_ordinality {
                    e.space();
                    e.token(TokenKind::FOR_KW);
                    e.space();
                    e.token(TokenKind::IDENT("ORDINALITY".to_string()));
                } else if let Some(ref type_name) = col.type_name {
                    e.space();
                    super::emit_type_name(e, type_name);

                    // Column path expression
                    if let Some(ref colexpr) = col.colexpr {
                        e.space();
                        e.token(TokenKind::IDENT("PATH".to_string()));
                        e.space();
                        super::emit_node(colexpr, e);
                    }

                    // Default expression
                    if let Some(ref defexpr) = col.coldefexpr {
                        e.space();
                        e.token(TokenKind::DEFAULT_KW);
                        e.space();
                        super::emit_node(defexpr, e);
                    }

                    if col.is_not_null {
                        e.space();
                        e.token(TokenKind::NOT_KW);
                        e.space();
                        e.token(TokenKind::NULL_KW);
                    }
                }
            }
        });
    }

    e.token(TokenKind::R_PAREN);

    // Alias (emit_alias includes the AS keyword)
    if let Some(ref alias) = n.alias {
        e.space();
        super::emit_alias(e, alias);
    }

    e.group_end();
}
