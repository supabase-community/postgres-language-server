//! The directives of Rust's `format!`, read like `std::fmt` does, see
//! https://doc.rust-lang.org/std/fmt/index.html#syntax
//!
//! ```text
//! format   := '{' [ argument ] [ ':' format_spec ] [ ws ] * '}'
//! argument := integer | identifier
//! ```
//!
//! `{{` and `}}` write `{` and `}`. The format spec isn't checked, it ends at the first `}`. A
//! `{` that doesn't start a directive, like in `'{"a": 1}'`, doesn't compile in Rust, so it is
//! SQL written without its escape: it stays as written.

use pgls_text_size::{TextRange, TextSize};

use crate::{Expansion, Value};

/// The directives of a format string, in text order.
pub(crate) struct Directives<'a> {
    text: &'a str,
    /// Where the next directive is looked for.
    pos: usize,
}

impl<'a> Directives<'a> {
    /// The longest format spec of a directive. A `{` with a longer one stays as written, so that
    /// reading a text with many `{` and no `}` takes linear time.
    const SPEC_LEN_MAX: usize = 256;

    pub(crate) fn new(text: &'a str) -> Self {
        Self { text, pos: 0 }
    }

    /// The end of the directive that starts at the `{` at `start`, and the name of its
    /// argument, if it is one. Identifiers are read as Unicode letters, digits and `_`, a
    /// superset of Rust's.
    fn directive(&self, start: usize) -> Option<(usize, &'a str)> {
        let rest = self.text.get(start + 1..)?;
        let argument_len = if rest.starts_with(|c: char| c.is_ascii_digit()) {
            rest.find(|c: char| !c.is_ascii_digit())
        } else if rest.starts_with(|c: char| c.is_alphabetic() || c == '_') {
            rest.find(|c: char| !c.is_alphanumeric() && c != '_')
        } else {
            Some(0)
        }?;
        let argument = rest.get(..argument_len)?;
        let mut after = rest.get(argument_len..)?;
        if let Some(spec) = after.strip_prefix(':') {
            let spec_len = spec
                .bytes()
                .take(Self::SPEC_LEN_MAX)
                .position(|b| b == b'{' || b == b'}')?;
            after = spec.get(spec_len..)?;
        } else {
            after = after.trim_start();
        }
        after.starts_with('}').then(|| {
            let end = self.text.len() - after.len() + 1;
            (end, argument)
        })
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
            } else if let Some((end, argument)) = rest
                .starts_with('{')
                .then(|| self.directive(start))
                .flatten()
            {
                (end, Value::Unknown(argument))
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
