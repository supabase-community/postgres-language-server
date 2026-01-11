use pgls_query::protobuf::RangeSubselect;

use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind, LineType};

pub(super) fn emit_range_subselect(e: &mut EventEmitter, n: &RangeSubselect) {
    e.group_start(GroupKind::RangeSubselect);

    if n.lateral {
        e.token(TokenKind::LATERAL_KW);
        e.space();
    }

    e.token(TokenKind::L_PAREN);
    e.line(LineType::Soft);
    e.indent_start();
    if let Some(ref subquery) = n.subquery {
        // Subqueries in FROM clause should not have semicolons
        if let Some(pgls_query::NodeEnum::SelectStmt(select)) = subquery.node.as_ref() {
            super::select_stmt::emit_select_stmt_no_semicolon(e, select);
        } else {
            super::emit_node(subquery, e);
        }
    }
    e.indent_end();
    e.line(LineType::Soft);
    e.token(TokenKind::R_PAREN);

    if let Some(ref alias) = n.alias {
        e.line(LineType::SoftOrSpace);
        super::emit_alias(e, alias);
    }

    e.group_end();
}
