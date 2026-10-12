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

/// Hint for what kind of dollar-quoted content we're emitting.
/// Used to pick a more meaningful delimiter tag.
#[derive(Debug, Clone, Copy)]
pub enum DollarQuoteHint {
    /// Function body - prefer $function$ or $body$
    Function,
    /// Procedure body - prefer $procedure$ or $body$
    Procedure,
    /// DO block - prefer $do$ or $body$
    Do,
}

/// Emits a function or `DO` body, whose string literal starts at or after `location`.
///
/// A body that still reads as it does in the source is written back exactly as it was quoted
/// there, so the tag of `$$ ... $$` and every byte in between survive formatting. A body that
/// changed, or whose source is unknown, is dollar-quoted verbatim: with the tag of the source
/// literal when the new body allows it, or with a tag picked from `hint`.
pub(super) fn emit_body_literal(
    e: &mut EventEmitter,
    value: &str,
    location: i32,
    hint: DollarQuoteHint,
) {
    let source_literal = e.source_string_literal_from(location);

    if let Some(literal) = source_literal
        && decode_literal(literal).as_deref() == Some(value)
    {
        e.token(TokenKind::STRING(literal.to_string()));
        return;
    }

    let delimiter = source_literal
        .and_then(dollar_quote_tag)
        .filter(|tag| can_dollar_quote(value, tag))
        .map(str::to_string)
        .unwrap_or_else(|| pick_dollar_delimiter(value, hint));
    e.token(TokenKind::DOLLAR_QUOTE_DELIMITER(delimiter.clone()));
    e.token(TokenKind::DOLLAR_QUOTE_BODY(value.to_string()));
    e.token(TokenKind::DOLLAR_QUOTE_DELIMITER(delimiter));
}

/// The value of a plain or dollar-quoted string literal, `None` for any other form.
fn decode_literal(literal: &str) -> Option<String> {
    if let Some(tag) = dollar_quote_tag(literal) {
        let body = literal.strip_prefix(tag)?.strip_suffix(tag)?;
        return Some(body.to_string());
    }

    let body = literal.strip_prefix('\'')?.strip_suffix('\'')?;
    Some(body.replace("''", "'"))
}

/// The opening tag of a dollar-quoted literal, `$$` or `$tag$`.
fn dollar_quote_tag(literal: &str) -> Option<&str> {
    let rest = literal.strip_prefix('$')?;
    let end = rest.find('$')?;
    Some(&literal[..end + 2])
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

/// Pick a dollar quote delimiter that doesn't conflict with the body content.
/// Uses the hint to prefer context-appropriate tags.
fn pick_dollar_delimiter(body: &str, hint: DollarQuoteHint) -> String {
    // Order tags based on the hint - put the most appropriate ones first
    let preferred_tags: &[&str] = match hint {
        DollarQuoteHint::Function => &[
            "$function$",
            "$body$",
            "$$",
            "$procedure$",
            "$do$",
            "$sql$",
            "$code$",
            "$_$",
        ],
        DollarQuoteHint::Procedure => &[
            "$procedure$",
            "$body$",
            "$$",
            "$function$",
            "$do$",
            "$sql$",
            "$code$",
            "$_$",
        ],
        DollarQuoteHint::Do => &[
            "$do$",
            "$body$",
            "$$",
            "$function$",
            "$procedure$",
            "$sql$",
            "$code$",
            "$_$",
        ],
    };

    for tag in preferred_tags {
        if can_dollar_quote(body, tag) {
            return (*tag).to_string();
        }
    }

    // Fall back to numbered tags if all preferred ones conflict
    let mut counter = 0usize;
    loop {
        let tag = format!("$f{counter}$");
        if can_dollar_quote(body, &tag) {
            return tag;
        }
        counter += 1;
    }
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
