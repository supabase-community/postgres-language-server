use pgt_query::protobuf::String as PgString;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

const RESERVED_KEYWORDS: &[&str] = &[
    "all",
    "analyse",
    "analyze",
    "and",
    "any",
    "array",
    "as",
    "asc",
    "asymmetric",
    "both",
    "case",
    "cast",
    "check",
    "collate",
    "column",
    "constraint",
    "create",
    "current_catalog",
    "current_date",
    "current_role",
    "current_time",
    "current_timestamp",
    "current_user",
    "default",
    "deferrable",
    "desc",
    "distinct",
    "do",
    "else",
    "end",
    "except",
    "false",
    "fetch",
    "for",
    "foreign",
    "from",
    "grant",
    "group",
    "having",
    "in",
    "initially",
    "intersect",
    "into",
    "lateral",
    "leading",
    "limit",
    "localtime",
    "localtimestamp",
    "not",
    "null",
    "offset",
    "on",
    "only",
    "or",
    "order",
    "placing",
    "primary",
    "references",
    "returning",
    "select",
    "session_user",
    "some",
    "symmetric",
    "system_user",
    "table",
    "then",
    "to",
    "trailing",
    "true",
    "union",
    "unique",
    "user",
    "using",
    "variadic",
    "when",
    "where",
    "window",
    "with",
];

pub(super) fn emit_string(e: &mut EventEmitter, n: &PgString) {
    e.group_start(GroupKind::String);
    emit_identifier_maybe_quoted(e, &n.sval);
    e.group_end();
}

pub(super) fn emit_string_literal(e: &mut EventEmitter, n: &PgString) {
    e.group_start(GroupKind::String);
    emit_single_quoted_str(e, &n.sval);
    e.group_end();
}

pub(super) fn emit_string_identifier(e: &mut EventEmitter, n: &PgString) {
    e.group_start(GroupKind::String);
    emit_identifier(e, &n.sval);
    e.group_end();
}

pub(super) fn emit_identifier(e: &mut EventEmitter, value: &str) {
    let escaped = value.replace('"', "\"\"");
    e.token(TokenKind::IDENT(format!("\"{}\"", escaped)));
}

/// Emit an identifier, adding quotes only if necessary.
/// Quotes are needed if the identifier:
/// - Contains special characters (space, punctuation, etc.)
/// - Is a SQL keyword
/// - Starts with a digit
/// - Contains uppercase letters (to preserve case)
/// Empty strings are ignored to match existing behaviour.
pub(super) fn emit_identifier_maybe_quoted(e: &mut EventEmitter, value: &str) {
    if value.is_empty() {
        return;
    }

    if needs_quoting(value) {
        emit_identifier(e, value);
    } else {
        e.token(TokenKind::IDENT(value.to_string()));
    }
}

pub(super) fn emit_keyword(e: &mut EventEmitter, keyword: &str) {
    if let Some(token) = TokenKind::from_keyword(keyword) {
        e.token(token);
    } else {
        e.token(TokenKind::IDENT(keyword.to_string()));
    }
}

pub(super) fn emit_single_quoted_str(e: &mut EventEmitter, value: &str) {
    let escaped = value.replace('\'', "''");
    e.token(TokenKind::STRING(format!("'{}'", escaped)));
}

pub(super) fn emit_dollar_quoted_str(e: &mut EventEmitter, value: &str) {
    let delimiter = pick_dollar_delimiter(value);
    e.token(TokenKind::DOLLAR_QUOTED_STRING(format!(
        "{}{}{}",
        delimiter, value, delimiter
    )));
}

fn needs_quoting(value: &str) -> bool {
    if value.is_empty() {
        return true;
    }

    let mut chars = value.chars();
    if let Some(first) = chars.next() {
        if first.is_ascii_digit() {
            return true;
        }
        if first == '_' && value.len() == 1 {
            return false;
        }
    }

    if value.chars().any(|c| c.is_ascii_uppercase()) {
        return true;
    }

    if value
        .chars()
        .any(|c| !c.is_ascii_alphanumeric() && c != '_')
    {
        return true;
    }

    let lower = value.to_ascii_lowercase();
    RESERVED_KEYWORDS.binary_search(&lower.as_str()).is_ok()
}

fn pick_dollar_delimiter(body: &str) -> String {
    if !body.contains("$$") {
        return "$$".to_string();
    }

    let mut counter = 0usize;
    loop {
        let tag = if counter == 0 {
            "$pg$".to_string()
        } else {
            format!("$pg{}$", counter)
        };

        if !body.contains(&tag) {
            return tag;
        }

        counter += 1;
    }
}
