use pgt_query::protobuf::RangeSubselect;

use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind};

pub(super) fn emit_range_subselect(e: &mut EventEmitter, n: &RangeSubselect) {
    e.group_start(GroupKind::RangeSubselect);

    if n.lateral {
        e.token(TokenKind::LATERAL_KW);
        e.space();
    }

    e.token(TokenKind::L_PAREN);
    if let Some(ref subquery) = n.subquery {
        // Subqueries in FROM clause should not have semicolons
        if let Some(pgt_query::NodeEnum::SelectStmt(select)) = subquery.node.as_ref() {
            super::select_stmt::emit_select_stmt_no_semicolon(e, select);
        } else {
            super::emit_node(subquery, e);
        }
    }
    e.token(TokenKind::R_PAREN);

    if let Some(ref alias) = n.alias {
        e.space();
        super::emit_alias(e, alias);
    }

    e.group_end();
}
