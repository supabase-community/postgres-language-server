use pgls_query::protobuf::String as PgString;

use crate::{
    TokenKind,
    codegen::token_kind::{KeywordCategory, keyword_category},
    emitter::{EventEmitter, GroupKind},
};

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
    e.token(TokenKind::IDENT(format!("\"{escaped}\"")));
}

/// Emit an identifier, adding quotes only if necessary.
/// Quotes are needed if the identifier:
/// - Contains special characters (space, punctuation, etc.)
/// - Is a SQL keyword
/// - Starts with a digit
/// - Contains uppercase letters (to preserve case)
///
/// Empty strings are ignored to match existing behaviour.
pub(super) fn emit_identifier_maybe_quoted(e: &mut EventEmitter, value: &str) {
    if value.is_empty() {
        return;
    }

    if needs_quoting_as_column_name(value) {
        emit_identifier(e, value);
    } else {
        e.token(TokenKind::IDENT(value.to_string()));
    }
}

/// Emits a type name part, honouring `type_case` when the name can be written unquoted.
///
/// A name that needs quotes keeps its own spelling: `"MyType"` and `"MYTYPE"` are two different
/// types in PostgreSQL, so casing it would change which type is referenced.
pub(super) fn emit_type_identifier_maybe_quoted(e: &mut EventEmitter, value: &str) {
    if value.is_empty() {
        return;
    }

    if needs_quoting(value) {
        emit_identifier(e, value);
    } else {
        e.token(TokenKind::TYPE_IDENT(value.to_string()));
    }
}

/// Emits a word the grammar reads case-insensitively in the keyword case. It need not be a
/// keyword: option and privilege names such as `usage` are plain words.
pub(super) fn emit_keyword(e: &mut EventEmitter, word: &str) {
    match TokenKind::from_keyword(word) {
        Some(keyword) => e.token(keyword),
        None => e.token(TokenKind::WORD(word.to_string())),
    }
}

/// Emits words separated by spaces, such as `ACCESS EXCLUSIVE`, in the keyword case.
pub(super) fn emit_keywords(e: &mut EventEmitter, words: &str) {
    for (index, word) in words.split(' ').enumerate() {
        if index > 0 {
            e.space();
        }
        emit_keyword(e, word);
    }
}

pub(super) fn emit_single_quoted_str(e: &mut EventEmitter, value: &str) {
    let escaped = value.replace('\'', "''");
    e.token(TokenKind::STRING(format!("'{escaped}'")));
}

/// Emits a function or `DO` body, dollar-quoted with `$$` so that every body in a file reads the
/// same, or with `$body$` when the body itself contains `$$`. Only the quotes change: the body is
/// written byte for byte, since Postgres stores it as the source of the function.
pub(super) fn emit_body_literal(e: &mut EventEmitter, value: &str) {
    let tag = dollar_quote_tag(value);
    e.token(TokenKind::DOLLAR_QUOTE_DELIMITER(tag.clone()));
    e.token(TokenKind::DOLLAR_QUOTE_BODY(value.to_string()));
    e.token(TokenKind::DOLLAR_QUOTE_DELIMITER(tag));
}

/// The first of `$$`, `$body$`, `$body1$`, `$body2$`, ... that can quote `body`.
fn dollar_quote_tag(body: &str) -> String {
    ["$$".to_string(), "$body$".to_string()]
        .into_iter()
        .chain((1..).map(|n| format!("$body{n}$")))
        .find(|tag| can_dollar_quote(body, tag))
        .expect("a numbered tag the body does not contain exists")
}

/// Whether `value` can be quoted with `tag`: the first occurrence of the tag after the opening one
/// must be the closing one.
fn can_dollar_quote(value: &str, tag: &str) -> bool {
    format!("{value}{tag}").find(tag) == Some(value.len())
}

pub(super) fn needs_quoting(value: &str) -> bool {
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

    keyword_category(value) == Some(KeywordCategory::Reserved)
}

/// Whether a schema, table or column name must be quoted. The grammar spells it as `ColId`, which
/// excludes the keywords that may name a function or type.
pub(super) fn needs_quoting_as_column_name(value: &str) -> bool {
    needs_quoting(value) || keyword_category(value) == Some(KeywordCategory::TypeFuncName)
}

/// Whether a name that may be any word, keywords included, must be quoted. The grammar spells it
/// as `ColLabel`, the trailing part of a qualified function name for instance.
pub(super) fn needs_quoting_as_label(value: &str) -> bool {
    needs_quoting(value) && keyword_category(value).is_none()
}

/// Whether an unqualified function or type name must be quoted.
///
/// The grammar spells such a name as `type_function_name`, which excludes column name keywords as
/// well as reserved ones: unquoted, `coalesce(a)` or `interval` would parse as the SQL construct of
/// the same name rather than as a reference to the named object.
pub(super) fn needs_quoting_as_function_or_type_name(value: &str) -> bool {
    needs_quoting(value) || keyword_category(value) == Some(KeywordCategory::ColName)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::{KeywordCase, RenderConfig, Renderer};

    fn render(value: &str, type_case: KeywordCase) -> String {
        let mut emitter = EventEmitter::new(Default::default());
        emit_type_identifier_maybe_quoted(&mut emitter, value);

        let mut output = String::new();
        let config = RenderConfig {
            type_case,
            ..Default::default()
        };
        let mut renderer = Renderer::new(&mut output, config);
        renderer.render(emitter.events).expect("render");
        output
    }

    #[test]
    fn an_unquoted_type_name_follows_the_type_case() {
        assert_eq!(render("object_id", KeywordCase::Upper), "OBJECT_ID");
        assert_eq!(render("object_id", KeywordCase::Lower), "object_id");
    }

    #[test]
    fn a_type_name_needing_quotes_keeps_its_own_case() {
        assert_eq!(render("MyType", KeywordCase::Upper), "\"MyType\"");
        assert_eq!(render("OBJECT_ID", KeywordCase::Lower), "\"OBJECT_ID\"");
    }
}
