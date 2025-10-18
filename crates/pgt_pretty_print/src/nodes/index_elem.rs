use pgt_query::protobuf::{IndexElem, SortByDir, SortByNulls};

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

pub(super) fn emit_index_elem(e: &mut EventEmitter, n: &IndexElem) {
    e.group_start(GroupKind::IndexElem);

    // Either a column name or an expression
    if let Some(ref expr) = n.expr {
        super::emit_node(expr, e);
    } else if !n.name.is_empty() {
        e.token(TokenKind::IDENT(n.name.clone()));
    }

    // Optional opclass
    if !n.opclass.is_empty() {
        e.space();
        super::node_list::emit_dot_separated_list(e, &n.opclass);
    }

    // Optional collation
    if !n.collation.is_empty() {
        e.space();
        e.token(TokenKind::COLLATE_KW);
        e.space();
        super::node_list::emit_dot_separated_list(e, &n.collation);
    }

    // Sort order (ASC/DESC)
    let ordering = SortByDir::try_from(n.ordering).unwrap_or(SortByDir::SortbyDefault);
    match ordering {
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
    let nulls_ordering =
        SortByNulls::try_from(n.nulls_ordering).unwrap_or(SortByNulls::SortbyNullsDefault);
    match nulls_ordering {
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
