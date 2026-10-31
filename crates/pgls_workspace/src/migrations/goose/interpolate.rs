//! The expansions that goose's `ENVSUB` replaces, read like goose does with [interpolate]. A
//! port of its parser, which follows this grammar:
//!
//! ```text
//! Expression = { Text | Expansion }
//! Text       = { "\\" | "\$" | "$$" | "$(" | all characters except "$" }
//! Expansion  = "$" ( Identifier | Brace )
//! Brace      = "{" Identifier [ Operation ] "}"
//! Operation  = ":-" Expression | "-" Expression | "?" Expression | ":" number [ ":" number ]
//! Identifier = letter { letter | digit | "_" }
//! ```
//!
//! An expression in a brace ends at the first `}`, so a `{` in a default is text.
//!
//! [interpolate]: https://github.com/mfridman/interpolate/blob/v0.0.2/parser.go

use std::ops::Range;

/// An expansion, like `$NAME`, `${NAME}` or `${NAME:-default}`.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct Expansion<'a> {
    /// The byte range of the expansion in its line.
    pub(super) range: Range<usize>,
    /// The name of its variable.
    pub(super) name: &'a str,
}

/// The expansions of a line, without those in the defaults of other expansions, or `None` if
/// interpolate rejects the line. goose fails on such a line. See
/// https://github.com/mfridman/interpolate/blob/v0.0.2/parser.go#L57
pub(super) fn expansions(line: &str) -> Option<Vec<Expansion<'_>>> {
    let mut expansions = Vec::new();
    Parser {
        input: line,
        pos: 0,
    }
    .expression(None, Some(&mut expansions))?;
    Some(expansions)
}

struct Parser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn rest(&self) -> &'a str {
        &self.input[self.pos..]
    }

    fn peek(&self) -> Option<char> {
        self.rest().chars().next()
    }

    fn next(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += c.len_utf8();
        Some(c)
    }

    fn scan_until(&mut self, stop: impl Fn(char) -> bool) -> &'a str {
        let start = self.pos;
        while let Some(c) = self.peek().filter(|c| !stop(*c)) {
            self.pos += c.len_utf8();
        }
        &self.input[start..self.pos]
    }

    /// An expression up to `stop` or the end of the line. Pushes its expansions to `expansions`
    /// if given.
    fn expression(
        &mut self,
        stop: Option<char>,
        mut expansions: Option<&mut Vec<Expansion<'a>>>,
    ) -> Option<()> {
        while let Some(c) = self.peek() {
            if Some(c) == stop {
                break;
            }
            // The escapes, and `$(`, which is left to the shell.
            if ["\\\\", "\\$", "$$", "$("]
                .iter()
                .any(|text| self.rest().starts_with(text))
            {
                self.pos += 2;
                continue;
            }
            // A `$` that isn't the last character starts an expansion.
            if c == '$' && self.pos < self.input.len() - 1 {
                let start = self.pos;
                let name = self.expansion()?;
                if let Some(expansions) = &mut expansions {
                    expansions.push(Expansion {
                        range: start..self.pos,
                        name,
                    });
                }
                continue;
            }
            self.next();
            self.scan_until(|c| c == '$' || c == '\\' || Some(c) == stop);
        }
        Some(())
    }

    /// An expansion, returning the name of its variable.
    fn expansion(&mut self) -> Option<&'a str> {
        self.next();
        if self.peek() == Some('{') {
            self.brace()
        } else {
            self.identifier()
        }
    }

    /// See https://github.com/mfridman/interpolate/blob/v0.0.2/parser.go#L128
    fn brace(&mut self) -> Option<&'a str> {
        self.next();
        let name = self.identifier()?;
        match self.next()? {
            '}' => return Some(name),
            ':' if self.peek() == Some('-') => {
                self.next();
                self.expression(Some('}'), None)?;
            }
            '-' | '?' => self.expression(Some('}'), None)?,
            // A substring, `${NAME:offset}` or `${NAME:offset:length}`.
            ':' => {
                number(self.scan_until(|c| c == ':' || c == '}'))?;
                if self.peek() == Some(':') {
                    self.next();
                    number(self.scan_until(|c| c == '}'))?;
                }
            }
            _ => return None,
        }
        (self.next()? == '}').then_some(name)
    }

    fn identifier(&mut self) -> Option<&'a str> {
        self.peek().filter(|c| c.is_alphabetic())?;
        Some(self.scan_until(|c| !c.is_alphabetic() && !c.is_numeric() && c != '_'))
    }
}

/// A number like Go's `strconv.Atoi` reads it, after trimming whitespace.
fn number(text: &str) -> Option<i64> {
    text.trim().parse().ok()
}

#[cfg(test)]
mod tests {
    use super::expansions;

    fn names(line: &str) -> Option<Vec<(&str, &str)>> {
        expansions(line).map(|expansions| {
            expansions
                .into_iter()
                .map(|expansion| (&line[expansion.range], expansion.name))
                .collect()
        })
    }

    /// The expected results are what interpolate v0.0.2 returns for these lines.
    #[test]
    fn follows_interpolate() {
        assert_eq!(
            names("create schema ${SCHEMA:-app{test};"),
            Some(vec![("${SCHEMA:-app{test}", "SCHEMA")])
        );
        assert_eq!(
            names("select ${A:-${B}}, ${C:-x}y}, ${D-}, ${E?msg}, ${F:1:2}, ${G: 1 }"),
            Some(vec![
                ("${A:-${B}}", "A"),
                ("${C:-x}", "C"),
                ("${D-}", "D"),
                ("${E?msg}", "E"),
                ("${F:1:2}", "F"),
                ("${G: 1 }", "G"),
            ])
        );
        assert_eq!(
            names("create table ${PREFIX}_events, app_$PREFIX, \"${SCHEMA}\""),
            Some(vec![
                ("${PREFIX}", "PREFIX"),
                ("$PREFIX", "PREFIX"),
                ("${SCHEMA}", "SCHEMA"),
            ])
        );
        // Escapes, a shell expansion, and a `$` at the end of the line are text.
        assert_eq!(
            names(r"select '$$', \${A}, \\, $(B), $C1_é$"),
            Some(vec![("$C1_é", "C1_é")])
        );
        // goose fails on these lines.
        for line in [
            "select ${A:?};",
            "select $1;",
            "select $A$ x;",
            "select ${ A};",
            "select ${A",
            "select ${A:-x",
            "select ${A:x}",
        ] {
            assert_eq!(names(line), None, "{line}");
        }
    }
}
