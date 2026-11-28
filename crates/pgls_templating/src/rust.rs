//! The directives of Rust's `format!`, read like `std::fmt` does, see
//! https://doc.rust-lang.org/std/fmt/index.html#syntax
//!
//! ```text
//! format      := '{' [ argument ] [ ':' format_spec ] [ ws ] * '}'
//! argument    := integer | identifier
//! format_spec := [[fill]align][sign]['#']['0'][width]['.' precision]type
//! fill        := character
//! align       := '<' | '^' | '>'
//! sign        := '+' | '-'
//! width       := count
//! precision   := count | '*'
//! type        := '' | '?' | 'x?' | 'X?' | identifier
//! count       := parameter | integer
//! parameter   := argument '$'
//! ```
//!
//! `{}` takes the next positional argument, `{0}` the argument at that position, and `{name}`
//! the argument of that name. A precision `.*` takes the next positional argument before the
//! value. `{{` and `}}` write `{` and `}`. A `{` that doesn't start a directive, like in
//! `'{"a": 1}'` or `{: a b }`, doesn't compile in Rust, so it is SQL written without its escape:
//! it stays as written.

use pgls_text_size::{TextRange, TextSize};

use crate::{Argument, Expansion, Value};

/// The directives of a format string, in text order.
pub(crate) struct Directives<'a> {
    text: &'a str,
    /// Where the next directive is looked for.
    pos: usize,
    /// The positional argument that the next `{}` takes, from 0.
    next: usize,
}

impl<'a> Directives<'a> {
    pub(crate) fn new(text: &'a str) -> Self {
        Self {
            text,
            pos: 0,
            next: 0,
        }
    }

    /// The end of the directive that starts at the `{` at `start`, the argument that it takes,
    /// and the positional argument that the next `{}` takes, if it is one. Identifiers are read
    /// as Unicode letters, digits and `_`, a superset of Rust's.
    fn directive(&self, start: usize) -> Option<(usize, Argument<'a>, usize)> {
        let rest = self.text.get(start + 1..)?;
        let argument_len = if rest.starts_with(|c: char| c.is_ascii_digit()) {
            rest.find(|c: char| !c.is_ascii_digit())
        } else if rest.starts_with(|c: char| c.is_alphabetic() || c == '_') {
            rest.find(|c: char| !c.is_alphanumeric() && c != '_')
        } else {
            Some(0)
        }?;
        let argument = rest.get(..argument_len)?;
        let mut next = self.next;
        let mut after = rest.get(argument_len..)?;
        if let Some(spec) = after.strip_prefix(':') {
            let (rest, star) = Self::format_spec(spec)?;
            if star {
                next = next.saturating_add(1);
            }
            after = rest;
        }
        after = after.trim_start();
        if !after.starts_with('}') {
            return None;
        }
        let argument = if argument.is_empty() {
            next = next.saturating_add(1);
            Argument::Index(next - 1)
        } else if let Ok(index) = argument.parse() {
            Argument::Index(index)
        } else {
            Argument::Named(argument)
        };
        Some((self.text.len() - after.len() + 1, argument, next))
    }
}

impl Directives<'_> {
    /// The text after the `format_spec` at the start of `spec`, and whether its precision is
    /// `*`, or `None` if it isn't one.
    fn format_spec(spec: &str) -> Option<(&str, bool)> {
        let mut rest = spec;
        // `[[fill]align]`: a fill is any character, followed by an align.
        let mut chars = rest.chars();
        let (first, second) = (chars.next(), chars.next());
        if second.is_some_and(|c| "<^>".contains(c)) {
            rest = rest.get(first?.len_utf8() + 1..)?;
        } else if first.is_some_and(|c| "<^>".contains(c)) {
            rest = rest.get(1..)?;
        }
        // `[sign]['#']['0']`.
        rest = rest.strip_prefix(['+', '-']).unwrap_or(rest);
        rest = rest.strip_prefix('#').unwrap_or(rest);
        rest = rest.strip_prefix('0').unwrap_or(rest);
        rest = Self::count(rest).unwrap_or(rest);
        let mut star = false;
        if let Some(precision) = rest.strip_prefix('.') {
            if let Some(after) = precision.strip_prefix('*') {
                star = true;
                rest = after;
            } else {
                rest = Self::count(precision)?;
            }
        }
        // `type`: an identifier, like `x` or `e`, and `?`.
        rest =
            Self::identifier(rest).map_or(Some(rest), |identifier| rest.get(identifier.len()..))?;
        rest = rest.strip_prefix('?').unwrap_or(rest);
        Some((rest, star))
    }

    /// The text after the `count` at the start of `text`: an integer, or an argument and `$`.
    fn count(text: &str) -> Option<&str> {
        let digits = text
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(text.len());
        if digits > 0 {
            let rest = text.get(digits..)?;
            return Some(rest.strip_prefix('$').unwrap_or(rest));
        }
        let identifier = Self::identifier(text)?;
        text.get(identifier.len()..)?.strip_prefix('$')
    }

    /// The identifier at the start of `text`.
    fn identifier(text: &str) -> Option<&str> {
        if !text.starts_with(|c: char| c.is_alphabetic() || c == '_') {
            return None;
        }
        let len = text
            .find(|c: char| !c.is_alphanumeric() && c != '_')
            .unwrap_or(text.len());
        text.get(..len)
    }
}

impl<'a> Iterator for Directives<'a> {
    type Item = Expansion<'a>;

    fn next(&mut self) -> Option<Expansion<'a>> {
        loop {
            let rest = self.text.get(self.pos..)?;
            let start = self.pos + rest.find(['{', '}'])?;
            let rest = self.text.get(start..)?;
            let (end, value) = if rest.starts_with("{{") {
                (start + 2, Value::Known("{"))
            } else if rest.starts_with("}}") {
                (start + 2, Value::Known("}"))
            } else if let Some((end, argument, next)) = rest
                .starts_with('{')
                .then(|| self.directive(start))
                .flatten()
            {
                self.next = next;
                (end, Value::Unknown { name: "", argument })
            } else {
                self.pos = start + 1;
                continue;
            };
            self.pos = end;
            // The directive is part of the text, so its offsets fit in a `TextSize`.
            let (Ok(start), Ok(end)) = (TextSize::try_from(start), TextSize::try_from(end)) else {
                return None;
            };
            return Some(Expansion {
                range: TextRange::new(start, end),
                value,
            });
        }
    }
}
