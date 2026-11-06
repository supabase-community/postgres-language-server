//! Parser proxies for SQL embedded in host-language format strings.
//!
//! Each unquoted template word becomes an opaque identifier padded to its source byte length. Keeping
//! the original spans lets diagnostics and fixes refer to the source rather than the proxies.
use std::{
    collections::{HashMap, HashSet},
    fmt,
    ops::Range,
    sync::LazyLock,
};

use pgls_tokenizer::{TokenKind, tokenize};
use regex::Regex;

use crate::{NamedParameterConversion, SyntaxKind, convert_to_positional_params_with_metadata};

// Large enough for generated migration files, while bounding scanner memory and work.
const SOURCE_BYTES_MAX: usize = 32 * 1024 * 1024;
const IDENTIFIERS_MAX: usize = 65_536;
const REPLACEMENTS_MAX: usize = 4_096;
const OCCURRENCES_MAX: usize = 1_048_576;
const MARKER_CANDIDATES_MAX: usize = 4_096;
// Format directives are normally a few bytes; this also bounds malformed format specifications.
const DIRECTIVE_BYTES_MAX: usize = 256;

fn push_proxy(output: &mut String, name: &str, source: &str) {
    debug_assert!(name.len() <= source.len());
    debug_assert!(source.is_char_boundary(name.len()));
    output.push_str(name);
    output.extend(source.bytes().skip(name.len()).map(|b| {
        if matches!(b, b'\n' | b'\r') {
            char::from(b)
        } else {
            ' '
        }
    }));
}

fn proxy_width(source: &str) -> usize {
    let limit = source.find(['\n', '\r']).unwrap_or(source.len()).min(6);
    source
        .char_indices()
        .map(|(index, _)| index)
        .chain(std::iter::once(source.len()))
        .take_while(|index| *index <= limit)
        .last()
        .unwrap_or(0)
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TemplateSyntax {
    pub go_printf: bool,
    pub rust_format: bool,
    pub dollar_values: bool,
}

#[derive(Clone, Debug)]
pub struct TemplateError {
    pub range: Range<usize>,
    message: &'static str,
}

impl fmt::Display for TemplateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.message)
    }
}

impl std::error::Error for TemplateError {}

fn limit_error(range: Range<usize>, message: &'static str) -> TemplateError {
    tracing::warn!(?range, message, "SQL template conversion limit reached");
    TemplateError { range, message }
}

#[expect(
    clippy::expect_used,
    reason = "The regex is a source constant, so an invalid pattern is a programmer error"
)]
static GO_DIRECTIVE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^%[+#\- 0]*(?:\[[1-9][0-9]*\])?(?:[0-9]+|\*)?(?:\.(?:\[[1-9][0-9]*\])?(?:[0-9]+|\*)?)?(?:\[[1-9][0-9]*\])?[vTtbcdoOqxXUeEfFgGsp]",
    )
    .expect("the Go printf directive regex is static")
});

#[expect(
    clippy::expect_used,
    reason = "The regex is a source constant, so an invalid pattern is a programmer error"
)]
static RUST_SPECIFICATION: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^(?:.[<^>]|[<^>])?[+-]?#?0?(?:[0-9]+\$?|[\p{XID_Start}_][\p{XID_Continue}]*\$)?(?:\.(?:[0-9]+\$?|[\p{XID_Start}_][\p{XID_Continue}]*\$|\*))?(?:[xX]\?|[?oxXpbEe])?[ \t\r\n]*$",
    )
    .expect("the Rust format specification regex is static")
});

fn directive_len(text: &str, syntax: TemplateSyntax) -> Option<usize> {
    match text.as_bytes().first() {
        Some(b'%') if syntax.go_printf => {}
        Some(b'{') if syntax.rust_format => {}
        _ => return None,
    }
    let end = text
        .char_indices()
        .map(|(offset, _)| offset)
        .take_while(|offset| *offset <= DIRECTIVE_BYTES_MAX)
        .last()
        .unwrap_or(0);
    let bounded = text.get(
        ..if text.len() <= DIRECTIVE_BYTES_MAX {
            text.len()
        } else {
            end
        },
    )?;
    match text.as_bytes().first() {
        Some(b'%') => GO_DIRECTIVE.find(bounded).map(|m| m.end()),
        Some(b'{') => {
            if !bounded.starts_with('{') || bounded.starts_with("{{") {
                return None;
            }
            let end = bounded.find('}')?;
            let body = bounded.get(1..end)?;
            if body.contains('{') {
                return None;
            }
            let argument = body.split(':').next()?;
            if let Some((_, specification)) = body.split_once(':')
                && !RUST_SPECIFICATION.is_match(specification)
            {
                return None;
            }
            let valid = argument.is_empty()
                || argument.chars().all(|c| c.is_ascii_digit())
                || argument
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_alphabetic() || c == '_')
                    && argument.chars().all(|c| c.is_alphanumeric() || c == '_');
            valid.then_some(end + 1)
        }
        _ => None,
    }
}

fn escaped_len(text: &str, syntax: TemplateSyntax) -> Option<usize> {
    match text.as_bytes().first() {
        Some(b'%') if syntax.go_printf && text.starts_with("%%") => Some(2),
        Some(b'{' | b'}')
            if syntax.rust_format && (text.starts_with("{{") || text.starts_with("}}")) =>
        {
            Some(2)
        }
        _ => None,
    }
}

/// Host-language directives, including those inside SQL strings and comments. Their order matters
/// to implicit Go/Rust argument indexes, so the formatter must preserve this sequence too.
pub fn template_parameters(text: &str, syntax: TemplateSyntax) -> impl Iterator<Item = &str> {
    let mut offset = 0;
    std::iter::from_fn(move || {
        if !syntax.go_printf && !syntax.rust_format || text.len() > SOURCE_BYTES_MAX {
            return None;
        }
        while let Some(tail) = text.get(offset..).filter(|tail| !tail.is_empty()) {
            if let Some(len) = escaped_len(tail, syntax) {
                offset += len;
                return tail.get(..len);
            } else if let Some(len) = directive_len(tail, syntax) {
                offset += len;
                return tail.get(..len);
            } else {
                offset += tail.chars().next().map_or(1, char::len_utf8);
            }
        }
        None
    })
}

fn word_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '_' | '$') || !c.is_ascii()
}

fn trivia(kind: TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Space
            | TokenKind::Tab
            | TokenKind::VerticalTab
            | TokenKind::FormFeed
            | TokenKind::LineEnding { .. }
            | TokenKind::LineComment
            | TokenKind::BlockComment { .. }
    )
}

fn ambiguous_tail(previous: Option<(TokenKind, &str)>) -> bool {
    previous.is_some_and(|(kind, text)| match kind {
        TokenKind::Ident => SyntaxKind::from_keyword(&text.to_ascii_lowercase()).is_none(),
        TokenKind::Unknown => text.chars().any(word_char),
        TokenKind::Literal { .. }
        | TokenKind::QuotedIdent { .. }
        | TokenKind::PositionalParam
        | TokenKind::NamedParam { .. }
        | TokenKind::CloseParen
        | TokenKind::CloseBracket => true,
        _ => false,
    })
}

/// The template-containing word at the start of `text`, including identifier prefixes/suffixes.
fn template_word_len(text: &str, syntax: TemplateSyntax) -> Option<usize> {
    let mut offset = 0;
    let mut has_template = false;
    while let Some(tail) = text.get(offset..).filter(|tail| !tail.is_empty()) {
        if escaped_len(tail, syntax).is_some() {
            break;
        }
        if let Some(len) = directive_len(tail, syntax) {
            offset += len;
            has_template = true;
        } else if let Some(c) = tail.chars().next().filter(|c| word_char(*c)) {
            offset += c.len_utf8();
        } else {
            break;
        }
    }
    has_template.then_some(offset)
}

fn marker(
    word_len: usize,
    reserved: &mut HashSet<String>,
    counters: &mut [usize; 7],
) -> Option<String> {
    // Two-byte directives have 26 * 36 possible proxies. Longer words get more room; they
    // cannot exhaust the pool reserved for short directives.
    let width = word_len.min(6);
    let next = counters.get_mut(width)?;
    for _ in 0..MARKER_CANDIDATES_MAX {
        let candidate = *next;
        *next += 1;
        let mut value = candidate;
        let mut suffix = String::new();
        for _ in 1..width {
            let digit = (value % 36) as u8;
            suffix.push(char::from(if digit < 10 {
                b'0' + digit
            } else {
                b'a' + digit - 10
            }));
            value /= 36;
        }
        if value >= 26 {
            break;
        }
        let name = format!("{}{suffix}", char::from(b'a' + value as u8));
        if SyntaxKind::from_keyword(&name).is_none() && reserved.insert(name.clone()) {
            return Some(name);
        }
    }
    None
}

/// Converts host-language placeholders and existing named parameters without rendering a template.
/// Template words are opaque identifiers/expressions, never arbitrary SQL clauses. SQL quotes,
/// comments and dollar-quoted bodies stay intact.
pub fn convert_template_parameters(
    text: &str,
    syntax: TemplateSyntax,
) -> Result<NamedParameterConversion, TemplateError> {
    if syntax == TemplateSyntax::default() {
        return Ok(convert_to_positional_params_with_metadata(text));
    }
    if text.len() > SOURCE_BYTES_MAX {
        return Err(limit_error(
            0..0,
            "SQL template exceeds the 32 MiB source limit",
        ));
    }
    let mut reserved = HashSet::new();
    if !syntax.go_printf && !syntax.rust_format {
        let mut conversion = convert_to_positional_params_with_metadata(text);
        protect_dollar_values(text, &mut conversion)?;
        return Ok(conversion);
    }
    let mut offset = 0;
    for token in tokenize(text) {
        let end = offset + token.len as usize;
        if matches!(
            token.kind,
            TokenKind::Ident | TokenKind::NamedParam { .. } | TokenKind::QuotedIdent { .. }
        ) && let Some(word) = text.get(offset..end)
        {
            reserved.insert(
                word.trim_start_matches([':', '@', '$'])
                    .trim_matches(['\'', '"'])
                    .to_lowercase(),
            );
            if reserved.len() > IDENTIFIERS_MAX {
                return Err(limit_error(
                    offset..end,
                    "Too many identifiers in SQL template",
                ));
            }
        }
        offset = end;
    }
    let mut normalized = String::with_capacity(text.len());
    let mut words = HashMap::<&str, String>::new();
    let mut replacements = HashMap::new();
    let mut symbols = HashMap::new();
    let mut ranges = Vec::new();
    let mut counters = [0; 7];
    let mut previous = None;
    offset = 0;
    while let Some(tail) = text.get(offset..).filter(|tail| !tail.is_empty()) {
        if let Some(len) = escaped_len(tail, syntax) {
            if tail.starts_with("%%") {
                // PostgreSQL's % and a custom %% operator have different precedence.
                normalized.push_str("% ");
                symbols.insert("%".to_string(), "%%".to_string());
                if ranges.len() >= OCCURRENCES_MAX {
                    return Err(limit_error(
                        offset..offset + len,
                        "Too many SQL template occurrences",
                    ));
                }
                ranges.push(offset..offset + len);
                previous = Some((TokenKind::Percent, "%"));
            } else {
                normalized.push_str(tail.get(..len).unwrap_or_default());
            }
            offset += len;
            continue;
        }
        let Some(token) = tokenize(tail).next() else {
            break;
        };
        let word_len = if token.kind == TokenKind::Ident
            || token.kind == TokenKind::Unknown && tail.chars().next().is_some_and(word_char)
            || directive_len(tail, syntax).is_some()
        {
            template_word_len(tail, syntax)
        } else {
            None
        };
        if let Some(len) = word_len {
            let end = offset + len;
            debug_assert!(text.is_char_boundary(offset));
            debug_assert!(text.is_char_boundary(end));
            let Some(word) = text.get(offset..end) else {
                break;
            };
            if ambiguous_tail(previous) {
                return Err(TemplateError {
                    range: offset..end,
                    message: "Ambiguous SQL template fragment. Use AS for a template alias; whole SQL clauses cannot be analysed as placeholders.",
                });
            }
            if ranges.len() >= OCCURRENCES_MAX {
                return Err(limit_error(
                    offset..end,
                    "Too many SQL template occurrences",
                ));
            }
            ranges.push(offset..end);
            let name = match words.get(word) {
                Some(name) => name.clone(),
                None => {
                    if words.len() >= REPLACEMENTS_MAX {
                        return Err(limit_error(
                            offset..end,
                            "Too many distinct SQL template placeholders",
                        ));
                    }
                    let width = proxy_width(word);
                    let name = marker(width, &mut reserved, &mut counters).ok_or_else(|| {
                        limit_error(
                            offset..end,
                            "No collision-free SQL template placeholder available",
                        )
                    })?;
                    replacements.insert(name.clone(), word.to_string());
                    words.insert(word, name.clone());
                    name
                }
            };
            // Preserve newlines inside Rust specifications as well as byte offsets.
            push_proxy(&mut normalized, &name, word);
            previous = Some((TokenKind::Ident, word));
            offset = end;
        } else {
            let len = token.len as usize;
            normalized.push_str(tail.get(..len).unwrap_or_default());
            if !trivia(token.kind) {
                previous = tail.get(..len).map(|text| (token.kind, text));
            }
            offset += len;
        }
    }
    debug_assert_eq!(normalized.len(), text.len());
    let mut conversion = convert_to_positional_params_with_metadata(&normalized);
    // Rendered names and expressions cannot be resolved or typechecked against a fixed catalog.
    conversion.has_identifier_parameters |= template_parameters(text, syntax).next().is_some();
    conversion.identifier_replacements.extend(replacements);
    conversion.symbol_replacements.extend(symbols);
    if syntax.dollar_values {
        protect_dollar_values(text, &mut conversion)?;
    }
    conversion
        .replacement_ranges
        .extend(ranges.into_iter().filter_map(|range| {
            range
                .start
                .try_into()
                .ok()
                .zip(range.end.try_into().ok())
                .map(|(start, end)| pgls_text_size::TextRange::new(start, end))
        }));
    conversion
        .replacement_ranges
        .sort_by_key(|range| range.start());
    Ok(conversion)
}

/// Give each directive a distinct identity for the formatter's AST round-trip check. Unlike the
/// parser proxies, these tags also cover quoted strings and comments: printf consumes arguments
/// there too. The tagged SQL is used only for comparison, never for diagnostics or printing.
pub fn bind_template_arguments(
    text: &str,
    syntax: TemplateSyntax,
) -> Result<String, TemplateError> {
    if text.len() > SOURCE_BYTES_MAX {
        return Err(limit_error(
            0..0,
            "SQL template exceeds the 32 MiB source limit",
        ));
    }
    let mut output = String::with_capacity(text.len());
    let mut reserved = HashSet::new();
    let mut counters = [0; 7];
    let mut end = 0;
    let mut occurrences = 0;
    let folded = text.to_ascii_lowercase();
    // Reserve every possible tag already present in the source, including string suffixes.
    // Five bounded-width scans avoid a full source search for each argument.
    for width in 2..=6 {
        for (start, window) in folded.as_bytes().windows(width).enumerate() {
            if window
                .iter()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
                && let Some(word) = folded.get(start..start + width)
                && !reserved.contains(word)
            {
                if reserved.len() >= IDENTIFIERS_MAX {
                    return Err(limit_error(
                        start..start + width,
                        "Too many static format argument tags",
                    ));
                }
                reserved.insert(word.to_string());
            }
        }
    }
    for parameter in template_parameters(text, syntax) {
        // Every parameter is a slice borrowed from this same allocation.
        let start = parameter.as_ptr() as usize - text.as_ptr() as usize;
        debug_assert!(text.is_char_boundary(start));
        debug_assert!(text.is_char_boundary(start + parameter.len()));
        output.push_str(text.get(end..start).unwrap_or_default());
        if escaped_len(parameter, syntax).is_some() {
            output.push_str(if parameter == "%%" { "% " } else { parameter });
        } else {
            occurrences += 1;
            if occurrences > REPLACEMENTS_MAX {
                return Err(limit_error(
                    start..start + parameter.len(),
                    "Too many format arguments to verify",
                ));
            }
            let name =
                marker(proxy_width(parameter), &mut reserved, &mut counters).ok_or_else(|| {
                    limit_error(
                        start..start + parameter.len(),
                        "No collision-free format argument tag available",
                    )
                })?;
            push_proxy(&mut output, &name, parameter);
        }
        end = start + parameter.len();
    }
    output.push_str(text.get(end..).unwrap_or_default());
    Ok(output)
}

/// PostgreSQL already parses $N as ParamRef. Keep its identity and type inference; only protect
/// these source spans from AST-derived edits, without interpreting strings or dollar-quoted bodies.
fn protect_dollar_values(
    text: &str,
    conversion: &mut NamedParameterConversion,
) -> Result<(), TemplateError> {
    let lexed = crate::lex(text);
    for (index, kind) in lexed.tokens().enumerate() {
        if kind == SyntaxKind::POSITIONAL_PARAM {
            let range = lexed.range(index);
            if conversion.replacement_ranges.len() >= OCCURRENCES_MAX {
                return Err(limit_error(
                    usize::from(range.start())..usize::from(range.end()),
                    "Too many dollar-value parameters",
                ));
            }
            conversion.replacement_ranges.push(range);
        }
    }
    conversion
        .replacement_ranges
        .sort_by_key(|range| range.start());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all_modes() -> TemplateSyntax {
        TemplateSyntax {
            go_printf: true,
            rust_format: true,
            dollar_values: true,
        }
    }

    #[test]
    fn template_mixed_syntax_preserves_offsets_and_native_bind_values() {
        let source =
            "SELECT %s, {value:>width$.precision$}, $123 FROM %s_{schema}.%[2]s WHERE id=$1;";
        let converted = convert_template_parameters(source, all_modes()).unwrap();
        assert_eq!(converted.sql.len(), source.len());
        assert!(converted.sql.contains("$123") && converted.sql.contains("$1"));
        for expected in ["%s", "{value:>width$.precision$}", "%s_{schema}", "%[2]s"] {
            assert!(
                converted
                    .identifier_replacements
                    .values()
                    .any(|word| word == expected),
                "{expected}"
            );
        }
        assert!(converted.value_replacements.is_empty());
    }

    #[test]
    fn template_dollar_values_preserve_native_nodes_and_only_protect_real_tokens() {
        let source = "SELECT $1, $2, $123, '$4', $$ $5 $$; -- $6\n";
        let converted = convert_template_parameters(
            source,
            TemplateSyntax {
                dollar_values: true,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(converted.sql, source);
        assert!(!converted.has_identifier_parameters);
        assert!(
            converted.identifier_replacements.is_empty() && converted.value_replacements.is_empty()
        );
        let protected: Vec<_> = converted
            .replacement_ranges
            .iter()
            .map(|range| &source[usize::from(range.start())..usize::from(range.end())])
            .collect();
        assert_eq!(protected, ["$1", "$2", "$123"]);
    }

    #[test]
    fn template_quotes_comments_and_dollar_bodies_remain_opaque() {
        let source = "SELECT '%s {name}', \"{schema}\", $$ %s {body} $$; -- %[1]s {}\n";
        let converted = convert_template_parameters(source, all_modes()).unwrap();
        assert_eq!(converted.sql, source);
        assert!(converted.identifier_replacements.is_empty());
        assert_eq!(
            template_parameters(source, all_modes()).collect::<Vec<_>>(),
            ["%s", "{name}", "{schema}", "%s", "{body}", "%[1]s", "{}"]
        );
    }

    #[test]
    fn template_escapes_are_not_arguments_and_go_modulo_has_native_precedence() {
        let source = "SELECT 1+2%%3, '{{}}', '%[3]*.[2]*[1]f';";
        let converted = convert_template_parameters(source, all_modes()).unwrap();
        assert_eq!(converted.sql, "SELECT 1+2% 3, '{{}}', '%[3]*.[2]*[1]f';");
        assert_eq!(
            converted.symbol_replacements.get("%").map(String::as_str),
            Some("%%")
        );
        assert_eq!(
            template_parameters(source, all_modes()).collect::<Vec<_>>(),
            ["%%", "{{", "}}", "%[3]*.[2]*[1]f"]
        );
    }

    #[test]
    fn template_unicode_prefixes_use_source_character_boundaries() {
        let source = "SELECT * FROM 漢éé_%s_{schema};";
        let converted = convert_template_parameters(source, all_modes()).unwrap();
        assert_eq!(converted.sql.len(), source.len());
        for (proxy, original) in &converted.identifier_replacements {
            assert!(original.is_char_boundary(proxy.len()));
        }
    }

    #[test]
    fn template_ambiguous_clause_fragments_fail_explicitly() {
        for source in ["SELECT * FROM items %s;", "SELECT 1 {alias};"] {
            let error = convert_template_parameters(source, all_modes()).unwrap_err();
            assert!(error.to_string().contains("Ambiguous"));
        }
        assert!(
            convert_template_parameters("SELECT * FROM items AS {alias};", all_modes()).is_ok()
        );
    }

    #[test]
    fn template_collisions_cannot_replace_a_static_identifier() {
        let converted = convert_template_parameters("SELECT a0, \"a1\", %s;", all_modes()).unwrap();
        assert!(!converted.identifier_replacements.contains_key("a0"));
        assert!(!converted.identifier_replacements.contains_key("a1"));
        let native = convert_template_parameters("SELECT $1, :value;", all_modes()).unwrap();
        assert!(!native.restorable);
    }

    #[test]
    fn template_argument_tags_distinguish_identical_implicit_fields() {
        let tagged = bind_template_arguments("SELECT '%s', '%s', a0;", all_modes()).unwrap();
        assert_ne!(tagged, "SELECT 'a0', 'a0', a0;");
        assert!(!tagged.contains("%s"));
        assert!(tagged.ends_with("a0;"));
        let literals: Vec<_> = tagged.split('\'').collect();
        assert_ne!(literals[1], literals[3]);
    }
}
