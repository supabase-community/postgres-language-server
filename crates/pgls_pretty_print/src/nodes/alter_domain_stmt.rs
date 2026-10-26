use pgls_query::protobuf::AlterDomainStmt;

use super::prelude::*;

pub(super) fn emit_alter_domain_stmt(e: &mut EventEmitter, n: &AlterDomainStmt) {
    e.group_start();

    e.token(TokenKind::ALTER_KW);
    e.space();
    e.token(TokenKind::DOMAIN_KW);
    e.space();

    // Note: missing_ok in AlterDomainStmt is for the constraint (DROP CONSTRAINT IF EXISTS),
    // not for the domain itself. ALTER DOMAIN doesn't support IF EXISTS at domain level.

    // Domain name (qualified)
    if !n.type_name.is_empty() {
        emit_dot_separated_list(e, &n.type_name);
    }

    // subtype field indicates the operation:
    // 'T' = SET DEFAULT, 'N' = DROP NOT NULL, 'O' = SET NOT NULL,
    // 'C' = ADD CONSTRAINT, 'X' = DROP CONSTRAINT, 'V' = VALIDATE CONSTRAINT
    // The operation is a continuation of the statement, indented like ALTER TABLE's commands.
    e.indent_start();
    e.line(LineType::SoftOrSpace);
    match n.subtype.as_str() {
        "T" => {
            // If def is None, it's DROP DEFAULT; otherwise SET DEFAULT value
            if let Some(ref def) = n.def {
                e.token(TokenKind::SET_KW);
                e.space();
                e.token(TokenKind::DEFAULT_KW);
                e.space();
                emit_node(e, def);
            } else {
                e.token(TokenKind::DROP_KW);
                e.space();
                e.token(TokenKind::DEFAULT_KW);
            }
        }
        "N" => {
            e.token(TokenKind::DROP_KW);
            e.space();
            e.token(TokenKind::NOT_KW);
            e.space();
            e.token(TokenKind::NULL);
        }
        "O" => {
            e.token(TokenKind::SET_KW);
            e.space();
            e.token(TokenKind::NOT_KW);
            e.space();
            e.token(TokenKind::NULL);
        }
        "C" => {
            e.token(TokenKind::ADD_KW);
            if let Some(ref def) = n.def {
                e.space();
                emit_node(e, def);
            }
        }
        "X" => {
            e.token(TokenKind::DROP_KW);
            e.space();
            e.token(TokenKind::CONSTRAINT_KW);
            if n.missing_ok {
                e.space();
                emit_if_exists(e);
            }
            if !n.name.is_empty() {
                e.space();
                emit_identifier_maybe_quoted(e, &n.name);
            }
            emit_drop_behavior(e, n.behavior(), Separator::Space, Restrict::Omitted);
        }
        "V" => {
            e.token(TokenKind::VALIDATE_KW);
            e.space();
            e.token(TokenKind::CONSTRAINT_KW);
            if !n.name.is_empty() {
                e.space();
                emit_identifier_maybe_quoted(e, &n.name);
            }
        }
        _ => {
            // Unknown subtype
        }
    }
    e.indent_end();

    e.group_end();
}
