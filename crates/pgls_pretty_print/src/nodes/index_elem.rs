use pgls_query::protobuf::{IndexElem, SortByDir, SortByNulls};

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
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
        super::emit_identifier_maybe_quoted(e, &n.name);
    }

    // The clauses after the column stay on its line: after a broken expression, they follow the
    // closing parenthesis instead of reading as a column of their own.

    // Optional collation
    if !n.collation.is_empty() {
        e.space();
        e.token(TokenKind::COLLATE_KW);
        e.space();
        super::node_list::emit_dot_separated_list(e, &n.collation);
    }

    // Optional opclass
    if !n.opclass.is_empty() {
        e.space();
        super::node_list::emit_dot_separated_list(e, &n.opclass);

        // Optional opclass options (e.g., tsvector_ops(siglen = 1000))
        if !n.opclassopts.is_empty() {
            super::node_list::emit_parenthesized_list(e, &n.opclassopts, |node, emitter| {
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
            e.space();
            e.token(TokenKind::NULLS_KW);
            e.space();
            e.token(TokenKind::FIRST_KW);
        }
        SortByNulls::SortbyNullsLast => {
            e.space();
            e.token(TokenKind::NULLS_KW);
            e.space();
            e.token(TokenKind::LAST_KW);
        }
        SortByNulls::SortbyNullsDefault | SortByNulls::Undefined => {}
    }

    e.group_end();
}
