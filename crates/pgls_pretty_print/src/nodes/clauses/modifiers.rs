//! The modifiers of DDL statements: `IF EXISTS`, `IF NOT EXISTS`, `OR REPLACE`, and `CASCADE` or
//! `RESTRICT`.

use pgls_query::protobuf::DropBehavior;

use crate::TokenKind;
use crate::emitter::EventEmitter;
use crate::nodes::utils::layout::{Separator, emit_separator};

pub(crate) fn emit_if_exists(e: &mut EventEmitter) {
    e.token(TokenKind::IF_KW);
    e.space();
    e.token(TokenKind::EXISTS_KW);
}

pub(crate) fn emit_if_not_exists(e: &mut EventEmitter) {
    e.token(TokenKind::IF_KW);
    e.space();
    e.token(TokenKind::NOT_KW);
    e.space();
    e.token(TokenKind::EXISTS_KW);
}

pub(crate) fn emit_or_replace(e: &mut EventEmitter) {
    e.token(TokenKind::OR_KW);
    e.space();
    e.token(TokenKind::REPLACE_KW);
}

/// Whether a statement writes `RESTRICT`, which is what it does without either keyword.
#[derive(Clone, Copy)]
pub(crate) enum Restrict {
    /// Only `CASCADE` is written.
    Omitted,
    /// `RESTRICT` is written too, when the statement has it.
    Written,
}

/// Emits `CASCADE` or `RESTRICT` after `separator`, when `behavior` and `restrict` ask for it.
pub(crate) fn emit_drop_behavior(
    e: &mut EventEmitter,
    behavior: DropBehavior,
    separator: Separator,
    restrict: Restrict,
) {
    match (behavior, restrict) {
        (DropBehavior::DropCascade, _) => {
            emit_separator(e, separator);
            e.token(TokenKind::CASCADE_KW);
        }
        (DropBehavior::DropRestrict, Restrict::Written) => {
            emit_separator(e, separator);
            e.token(TokenKind::RESTRICT_KW);
        }
        (DropBehavior::DropRestrict, Restrict::Omitted) | (DropBehavior::Undefined, _) => {}
    }
}
