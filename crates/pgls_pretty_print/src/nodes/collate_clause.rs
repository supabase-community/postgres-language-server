use pgls_query::{NodeEnum, protobuf::CollateClause};

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

use super::node_list::emit_dot_separated_list_with;

pub(super) fn emit_collate_clause(e: &mut EventEmitter, n: &CollateClause) {
    e.group_start(GroupKind::CollateClause);

    // Emit the argument being collated
    // COLLATE has high precedence, so we need parentheses around complex expressions
    if let Some(ref arg) = n.arg {
        let needs_parens = matches!(
            arg.node.as_ref(),
            Some(
                NodeEnum::AExpr(_)
                    | NodeEnum::BoolExpr(_)
                    | NodeEnum::SubLink(_)
                    | NodeEnum::CoalesceExpr(_)
                    | NodeEnum::CaseExpr(_)
            )
        );

        if needs_parens {
            e.token(TokenKind::L_PAREN);
        }
        super::emit_node(arg, e);
        if needs_parens {
            e.token(TokenKind::R_PAREN);
        }
        e.space();
    }

    e.token(TokenKind::COLLATE_KW);
    e.space();

    // Emit the collation name (qualified name)
    // Must quote to preserve case (PostgreSQL lowercases unquoted identifiers)
    emit_dot_separated_list_with(e, &n.collname, |node, e| {
        if let Some(pgls_query::NodeEnum::String(s)) = &node.node {
            super::emit_string_identifier(e, s);
        }
    });

    e.group_end();
}
