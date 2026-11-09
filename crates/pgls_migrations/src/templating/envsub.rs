//! Replacing the expansions of goose's `ENVSUB` with placeholders that parse.

use pgls_lexer::SyntaxKind;
use pgls_text_size::{TextRange, TextSize};

use super::Preprocessed;

/// An `ENVSUB` expansion in the file.
pub(crate) struct Expansion<'a> {
    pub(crate) range: TextRange,
    pub(crate) name: &'a str,
}

/// Replaces the identifiers with `expansions` with named parameters, see
/// [`crate::Goose`].
pub(crate) fn substitute(text: &str, expansions: &[Expansion]) -> Preprocessed {
    // Lex the text with each expansion as an identifier of the same length, to find the tokens
    // that the expansions are part of.
    let mut filled = String::with_capacity(text.len());
    let mut end = 0;
    for expansion in expansions {
        let range = std::ops::Range::<usize>::from(expansion.range);
        filled.push_str(&text[end..range.start]);
        filled.extend(std::iter::repeat_n('x', range.len()));
        end = range.end;
    }
    filled.push_str(&text[end..]);
    let lexed = pgls_lexer::lex(&filled);

    // The identifiers with expansions, extended to whole expansions, and the expansions in
    // other tokens than identifiers and comments, like string literals.
    let mut identifiers: Vec<TextRange> = Vec::new();
    let mut in_values: Vec<TextRange> = Vec::new();
    for (index, kind) in lexed.tokens().enumerate() {
        if matches!(kind, SyntaxKind::COMMENT | SyntaxKind::EOF) {
            continue;
        }
        let token = lexed.range(index);
        let first = expansions.partition_point(|expansion| expansion.range.end() <= token.start());
        let in_token = expansions[first..]
            .iter()
            .take_while(|expansion| expansion.range.start() < token.end());
        if kind != SyntaxKind::IDENT {
            in_values.extend(in_token.map(|expansion| expansion.range));
            continue;
        }
        let Some(range) = in_token.fold(None, |range: Option<TextRange>, expansion| {
            Some(range.unwrap_or(token).cover(expansion.range))
        }) else {
            continue;
        };
        match identifiers.last_mut() {
            Some(last) if last.end() > range.start() => *last = last.cover(range),
            _ => identifiers.push(range),
        }
    }

    let mut substituted = String::with_capacity(text.len());
    let mut placeholders = Vec::new();
    let mut unknown_values = Vec::new();
    let mut end = TextSize::from(0);
    let mut expansions = expansions.iter().peekable();
    for identifier in identifiers {
        substituted.push_str(&text[TextRange::new(end, identifier.start())]);
        // Expansions outside of identifiers stay as written.
        while let Some(expansion) =
            expansions.next_if(|expansion| expansion.range.start() < identifier.start())
        {
            placeholders.push(expansion.range);
            if in_values.contains(&expansion.range) {
                unknown_values.push(expansion.range);
            }
        }
        let mut name = String::new();
        let mut written = identifier.start();
        while let Some(expansion) =
            expansions.next_if(|expansion| expansion.range.start() < identifier.end())
        {
            name.extend(name_chars(
                &text[TextRange::new(written, expansion.range.start())],
            ));
            name.extend(name_chars(expansion.name));
            written = expansion.range.end();
        }
        name.extend(name_chars(&text[TextRange::new(written, identifier.end())]));
        if !name.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_') {
            name.insert(0, 'e');
        }
        let mut parameter = format!("${name}");
        let len = usize::from(identifier.len());
        parameter.truncate(len);
        substituted.push_str(&parameter);
        substituted.extend(std::iter::repeat_n('_', len - parameter.len()));
        placeholders.push(identifier);
        end = identifier.end();
    }
    substituted.push_str(&text[usize::from(end)..]);
    for expansion in expansions {
        placeholders.push(expansion.range);
        if in_values.contains(&expansion.range) {
            unknown_values.push(expansion.range);
        }
    }

    debug_assert_eq!(substituted.len(), text.len());
    Preprocessed {
        text: substituted,
        placeholders,
        unknown_values,
    }
}

/// The characters of `text` that a named parameter can hold.
fn name_chars(text: &str) -> impl Iterator<Item = char> + '_ {
    text.chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
}
