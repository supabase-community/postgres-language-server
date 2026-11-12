use pgls_query::protobuf::AtAlterConstraint;

use super::prelude::*;

/// Emits `CONSTRAINT name` and the attributes an `ALTER CONSTRAINT` changes.
pub(super) fn emit_atalter_constraint(e: &mut EventEmitter, c: &AtAlterConstraint) {
    e.token(TokenKind::CONSTRAINT_KW);
    e.space();
    emit_identifier_maybe_quoted(e, &c.conname);

    if c.alter_enforceability {
        e.space();
        if !c.is_enforced {
            e.token(TokenKind::NOT_KW);
            e.space();
        }
        e.token(TokenKind::ENFORCED_KW);
    }

    if c.alter_deferrability {
        // DEFERRABLE or NOT DEFERRABLE
        e.space();
        if !c.deferrable {
            e.token(TokenKind::NOT_KW);
            e.space();
        }
        e.token(TokenKind::DEFERRABLE_KW);

        // INITIALLY DEFERRED or INITIALLY IMMEDIATE
        e.space();
        e.token(TokenKind::INITIALLY_KW);
        e.space();
        if c.initdeferred {
            e.token(TokenKind::DEFERRED_KW);
        } else {
            e.token(TokenKind::IMMEDIATE_KW);
        }
    }

    if c.alter_inheritability {
        e.space();
        if c.noinherit {
            e.token(TokenKind::NO_KW);
            e.space();
        }
        e.token(TokenKind::INHERIT_KW);
    }
}
