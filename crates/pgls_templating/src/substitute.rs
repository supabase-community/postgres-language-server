//! Replacing the expansions of a host with placeholders that parse.

use pgls_lexer::SyntaxKind;
use pgls_text_size::{TextRange, TextSize};

use crate::{Ambiguity, Expansion, Preprocessed, Value};

impl Preprocessed {
    /// The text that the analysis reads for `text`, with placeholders for `expansions`, which
    /// are in file order and don't overlap.
    ///
    /// We don't know the values, so an identifier with expansions, like `${SCHEMA}`,
    /// `${PREFIX}_events`, `"${SCHEMA}"` or `app_%s`, becomes the named parameter `$SCHEMA`,
    /// `$PREFIX_events`, `$SCHEMA` or `$app_s`, padded with `_` to the same length, so that the
    /// whole identifier is one token with room for the index of a parameter. Named parameters
    /// are analysed like psql variables: as values, like in `where id = %d`, or as identifiers
    /// that aren't typechecked, like in `from %s.%s`. A [`Value::Known`] outside of identifiers
    /// and strings, like Go's `%%` in `a %% b`, becomes its text, padded with spaces. Expansions
    /// in string literals and comments stay as written, and those outside comments are
    /// [`Preprocessed::unknown_values`].
    ///
    /// Cost: one lex of the text, and one pass over the tokens and the expansions.
    pub fn substitute(text: &str, expansions: &[Expansion]) -> Preprocessed {
        Self::substitute_reading(text, expansions, |_, _| false)
    }

    /// [`Preprocessed::substitute`], with the identifiers with expansions that directly follow a
    /// token for which `ambiguous_after` holds in [`Preprocessed::ambiguous`].
    pub(crate) fn substitute_reading(
        text: &str,
        expansions: &[Expansion],
        ambiguous_after: impl Fn(SyntaxKind, &str) -> bool,
    ) -> Preprocessed {
        // Lex the text with each unknown value as an identifier of the same length, and each
        // known one as its text, to find the tokens that the expansions are part of.
        let mut filled = String::with_capacity(text.len());
        let mut end = 0;
        for expansion in expansions {
            let range = std::ops::Range::<usize>::from(expansion.range);
            filled.push_str(&text[end..range.start]);
            match expansion.value {
                Value::Unknown(_) => filled.extend(std::iter::repeat_n('x', range.len())),
                Value::Known(written) => {
                    debug_assert!(written.len() <= range.len());
                    filled.push_str(written);
                    filled.extend(std::iter::repeat_n(
                        ' ',
                        range.len().saturating_sub(written.len()),
                    ));
                }
            }
            end = range.end;
        }
        filled.push_str(&text[end..]);
        let lexed = pgls_lexer::lex(&filled);

        // The identifiers with expansions, extended to whole expansions, and where each
        // expansion is.
        let mut identifiers: Vec<TextRange> = Vec::new();
        let mut places = vec![Place::AsWritten; expansions.len()];
        let mut ambiguous = Vec::new();
        // The token before the current one, without trivia.
        let mut previous: Option<(SyntaxKind, TextRange)> = None;
        for (index, kind) in lexed.tokens().enumerate() {
            if kind.is_trivia() || kind == SyntaxKind::EOF {
                continue;
            }
            let token = lexed.range(index);
            let before = previous.replace((kind, token));
            let first =
                expansions.partition_point(|expansion| expansion.range.end() <= token.start());
            let in_token = expansions
                .get(first..)
                .unwrap_or_default()
                .iter()
                .take_while(|expansion| expansion.range.start() < token.end())
                .count();
            let in_token = first..first + in_token;
            if kind != SyntaxKind::IDENT {
                let in_string = matches!(
                    kind,
                    SyntaxKind::STRING
                        | SyntaxKind::ESC_STRING
                        | SyntaxKind::BYTE_STRING
                        | SyntaxKind::BIT_STRING
                        | SyntaxKind::DOLLAR_QUOTED_STRING
                );
                for (expansion, place) in expansions
                    .get(in_token.clone())
                    .unwrap_or_default()
                    .iter()
                    .zip(places.get_mut(in_token).unwrap_or_default())
                {
                    *place = match expansion.value {
                        Value::Known(_) if !in_string => Place::Written,
                        _ => Place::Value,
                    };
                }
                continue;
            }
            let in_token = expansions.get(in_token).unwrap_or_default();
            let Some(range) = in_token
                .iter()
                .fold(None, |range: Option<TextRange>, expansion| {
                    Some(range.unwrap_or(token).cover(expansion.range))
                })
            else {
                continue;
            };
            // Only a value of the host makes the identifier ambiguous, not a `%%` in a quoted
            // name.
            let has_unknown = in_token
                .iter()
                .any(|expansion| matches!(expansion.value, Value::Unknown(_)));
            if let Some((before_kind, before)) = before
                && has_unknown
                && ambiguous_after(before_kind, &filled[before])
            {
                ambiguous.push(Ambiguity {
                    placeholder: range,
                    after: before,
                });
            }
            match identifiers.last_mut() {
                Some(last) if last.end() > range.start() => *last = last.cover(range),
                _ => identifiers.push(range),
            }
        }

        let mut substitution = Substitution {
            text,
            filled: &filled,
            substituted: String::with_capacity(text.len()),
            placeholders: Vec::new(),
            unknown_values: Vec::new(),
            ambiguous,
            end: TextSize::from(0),
        };
        let mut next = 0;
        for identifier in identifiers {
            // Expansions outside of identifiers.
            while let Some((expansion, place)) = expansions
                .get(next)
                .zip(places.get(next))
                .filter(|(expansion, _)| expansion.range.start() < identifier.start())
            {
                substitution.outside(expansion.range, *place);
                next += 1;
            }
            let mut name = Parameter::default();
            let mut written = identifier.start();
            while let Some(expansion) = expansions
                .get(next)
                .filter(|expansion| expansion.range.start() < identifier.end())
            {
                name.push(&text[TextRange::new(written, expansion.range.start())]);
                name.push(expansion.value.name());
                written = expansion.range.end();
                next += 1;
            }
            name.push(&text[TextRange::new(written, identifier.end())]);
            substitution.identifier(identifier, name);
        }
        for (expansion, place) in expansions
            .get(next..)
            .unwrap_or_default()
            .iter()
            .zip(places.get(next..).unwrap_or_default())
        {
            substitution.outside(expansion.range, *place);
        }
        substitution.finish()
    }
}

/// Where an expansion is, which decides what the text holds there.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Place {
    /// In a comment: the text holds it as written.
    AsWritten,
    /// In a value, like a string literal: the text holds it as written, and it is an unknown
    /// value.
    Value,
    /// A known value outside of strings: the text holds what the host writes.
    Written,
}

/// The text of a [`Preprocessed`] while it is written.
struct Substitution<'a> {
    text: &'a str,
    /// The text with each expansion filled in, see [`Preprocessed::substitute`].
    filled: &'a str,
    substituted: String,
    placeholders: Vec<TextRange>,
    unknown_values: Vec<TextRange>,
    ambiguous: Vec<Ambiguity>,
    /// Where the text is written up to.
    end: TextSize,
}

impl Substitution<'_> {
    /// Copies the text as written up to `offset`.
    fn copy_to(&mut self, offset: TextSize) {
        self.substituted
            .push_str(&self.text[TextRange::new(self.end, offset)]);
        self.end = offset;
    }

    /// An expansion outside of identifiers.
    fn outside(&mut self, range: TextRange, place: Place) {
        match place {
            Place::AsWritten => {}
            Place::Value => self.unknown_values.push(range),
            Place::Written => {
                self.copy_to(range.start());
                self.substituted.push_str(&self.filled[range]);
                self.end = range.end();
            }
        }
        self.placeholders.push(range);
    }

    /// An identifier with expansions, which becomes the named parameter `name`.
    fn identifier(&mut self, identifier: TextRange, name: Parameter) {
        self.copy_to(identifier.start());
        self.substituted
            .push_str(&name.padded(usize::from(identifier.len())));
        self.end = identifier.end();
        self.placeholders.push(identifier);
    }

    fn finish(mut self) -> Preprocessed {
        self.copy_to(TextSize::of(self.text));
        debug_assert_eq!(self.substituted.len(), self.text.len());
        Preprocessed {
            text: self.substituted,
            placeholders: self.placeholders,
            unknown_values: self.unknown_values,
            ambiguous: self.ambiguous,
        }
    }
}

/// The name of the named parameter that replaces an identifier with expansions.
#[derive(Default)]
struct Parameter(String);

impl Parameter {
    /// Adds the characters of `text` that a named parameter can hold.
    fn push(&mut self, text: &str) {
        self.0.extend(
            text.chars()
                .filter(|c| c.is_ascii_alphanumeric() || *c == '_'),
        );
    }

    /// The parameter, `$name`, cut or padded with `_` to `len` bytes.
    fn padded(mut self, len: usize) -> String {
        if !self
            .0
            .starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
        {
            self.0.insert(0, 'e');
        }
        let mut parameter = format!("${}", self.0);
        parameter.truncate(len);
        parameter.extend(std::iter::repeat_n('_', len - parameter.len()));
        parameter
    }
}
