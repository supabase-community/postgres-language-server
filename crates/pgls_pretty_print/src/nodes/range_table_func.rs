use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
    nodes::node_list::emit_comma_separated_list,
};
use pgls_query::{NodeEnum, protobuf::RangeTableFunc};

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
        emit_comma_separated_list(e, &n.columns, |node, emitter| {
            if let Some(NodeEnum::RangeTableFuncCol(col)) = node.node.as_ref() {
                super::range_table_func_col::emit_range_table_func_col(emitter, col);
            } else {
                super::emit_node(node, emitter);
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
