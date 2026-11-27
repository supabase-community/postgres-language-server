//! The directives of Go's `fmt.Sprintf`, read like `doPrintf` does, see
//! https://pkg.go.dev/fmt#hdr-Printing and
//! https://github.com/golang/go/blob/go1.25.0/src/fmt/print.go
//!
//! ```text
//! Directive = "%" { Flag } [ Index ] [ Width ] [ "." [ Index ] [ Count ] ] [ Index ] Verb
//! Flag      = "+" | "-" | "#" | " " | "0"
//! Index     = "[" digits "]"
//! Width     = "*" | digits
//! Count     = "*" | digits
//! ```
//!
//! A directive with the verb `%`, like `%%`, writes `%`. For any other verb than those of `fmt`,
//! like the `% 2,` of `7 % 2, 8`, Go writes an error into the text, so such a `%` is the modulo
//! operator written without its escape: it stays as written.

use pgls_text_size::{TextRange, TextSize};

use crate::{Expansion, Value};

/// The directives of a format string, in text order.
pub(crate) struct Directives<'a> {
    text: &'a str,
    /// Where the next directive is looked for.
    pos: usize,
}

impl<'a> Directives<'a> {
    pub(crate) fn new(text: &'a str) -> Self {
        Self { text, pos: 0 }
    }

    /// The end of the directive that starts at the `%` at `start`, and what Go writes there, if
    /// it is one.
    fn directive(&self, start: usize) -> Option<(usize, Value<'a>)> {
        let bytes = self.text.as_bytes();
        let mut i = start + 1;
        while matches!(bytes.get(i), Some(b'+' | b'-' | b'#' | b' ' | b'0')) {
            i += 1;
        }
        i = self.index(i)?;
        i = self.count(i);
        // Go reads a `.` at the end as the verb.
        if bytes.get(i) == Some(&b'.') && i + 1 < bytes.len() {
            i = self.index(i + 1)?;
            i = self.count(i);
        }
        i = self.index(i)?;
        let value = match bytes.get(i)? {
            b'%' => Value::Known("%"),
            b'v' | b'T' | b't' | b'b' | b'c' | b'd' | b'o' | b'O' | b'q' | b'x' | b'X' | b'U'
            | b'e' | b'E' | b'f' | b'F' | b'g' | b'G' | b's' | b'p' => {
                Value::Unknown(self.text.get(i..i + 1)?)
            }
            _ => return None,
        };
        Some((i + 1, value))
    }

    /// The end of the argument index `[n]` at `i`, `i` without one, or `None` if it is malformed.
    fn index(&self, i: usize) -> Option<usize> {
        let bytes = self.text.as_bytes();
        if bytes.get(i) != Some(&b'[') {
            return Some(i);
        }
        let digits = self.digits(i + 1);
        (digits > i + 1 && bytes.get(digits) == Some(&b']')).then_some(digits + 1)
    }

    /// The end of the width or precision at `i`: `*` or digits.
    fn count(&self, i: usize) -> usize {
        if self.text.as_bytes().get(i) == Some(&b'*') {
            i + 1
        } else {
            self.digits(i)
        }
    }

    /// The end of the digits at `i`.
    fn digits(&self, i: usize) -> usize {
        let bytes = self.text.as_bytes();
        let mut end = i;
        while bytes.get(end).is_some_and(u8::is_ascii_digit) {
            end += 1;
        }
        end
    }
}

impl<'a> Iterator for Directives<'a> {
    type Item = Expansion<'a>;

    fn next(&mut self) -> Option<Expansion<'a>> {
        loop {
            let start = self.pos + self.text.get(self.pos..)?.find('%')?;
            let Some((end, value)) = self.directive(start) else {
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
