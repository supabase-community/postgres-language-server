use pgls_query::protobuf::{IndexElem, SortByDir, SortByNulls};

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
};

pub(super) fn emit_index_elem(e: &mut EventEmitter, n: &IndexElem) {
    e.group_start(GroupKind::IndexElem);

    // Either a column name or an expression
    if let Some(ref expr) = n.expr {
        // Expressions in index definitions must be wrapped in parentheses
        e.token(TokenKind::L_PAREN);
        super::emit_node(expr, e);
        e.token(TokenKind::R_PAREN);
    } else if !n.name.is_empty() {
        e.token(TokenKind::IDENT(n.name.clone()));
    }

    // Optional collation
    if !n.collation.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::COLLATE_KW);
        e.space();
        super::node_list::emit_dot_separated_list(e, &n.collation);
    }

    // Optional opclass
    if !n.opclass.is_empty() {
        e.line(LineType::SoftOrSpace);
        super::node_list::emit_dot_separated_list(e, &n.opclass);

        // Optional opclass options (e.g., tsvector_ops(siglen = 1000))
        if !n.opclassopts.is_empty() {
            e.token(TokenKind::L_PAREN);
            super::node_list::emit_comma_separated_list(e, &n.opclassopts, |node, emitter| {
                if let Some(pgls_query::NodeEnum::DefElem(def)) = node.node.as_ref() {
                    emitter.token(TokenKind::IDENT(def.defname.clone()));
                    if let Some(ref arg) = def.arg {
                        emitter.space();
                        emitter.token(TokenKind::IDENT("=".to_string()));
                        emitter.space();
                        super::emit_node(arg, emitter);
                    }
                }
            });
            e.token(TokenKind::R_PAREN);
        }
    }

    // Sort order (ASC/DESC)
    match n.ordering() {
        SortByDir::SortbyAsc => {
            e.space();
            e.token(TokenKind::ASC_KW);
        }
        SortByDir::SortbyDesc => {
            e.space();
            e.token(TokenKind::DESC_KW);
        }
        SortByDir::SortbyDefault | SortByDir::SortbyUsing | SortByDir::Undefined => {}
    }

    // NULLS FIRST/LAST
    match n.nulls_ordering() {
        SortByNulls::SortbyNullsFirst => {
            e.line(LineType::SoftOrSpace);
            e.token(TokenKind::NULLS_KW);
            e.space();
            e.token(TokenKind::FIRST_KW);
        }
        SortByNulls::SortbyNullsLast => {
            e.line(LineType::SoftOrSpace);
            e.token(TokenKind::NULLS_KW);
            e.space();
            e.token(TokenKind::LAST_KW);
        }
        SortByNulls::SortbyNullsDefault | SortByNulls::Undefined => {}
    }

    e.group_end();
}
