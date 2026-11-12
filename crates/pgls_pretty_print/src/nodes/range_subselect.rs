use pgls_query::protobuf::RangeSubselect;

use super::prelude::*;

pub(super) fn emit_range_subselect(e: &mut EventEmitter, n: &RangeSubselect) {
    e.group_start();

    if n.lateral {
        e.token(TokenKind::LATERAL_KW);
        e.space();
    }

    e.token(TokenKind::L_PAREN);
    e.line(LineType::Soft);
    e.indent_start();
    if let Some(ref subquery) = n.subquery {
        emit_node(e, subquery);
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
