//! The lexer compared with Postgres' scanner, for the lexer's tests, the corpus check and the fuzz
//! target `lexer_vs_scan`.

use pgls_text_size::{TextRange, TextSize};

use crate::{Lexed, SyntaxKind};

/// The characters of Postgres' operators (`op_chars` in `scan.l`), and `:` of `::` and `:=`.
///
/// https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/scan.l#L367
const OPERATOR_CHARS: &str = "~!@#^&|`?+-*/%<>=:";

impl Lexed<'_> {
    /// Where the tokens first differ from those of Postgres' scanner (`pgls_query::scan`), or
    /// `None` if they agree, the scanner rejects the text, or the text has psql meta-commands or
    /// COPY data, which the scanner reads as SQL. Whitespace and operators aren't compared, and named
    /// parameters (`:name`, `@name`, `$name`) are compared without their prefix, which the
    /// scanner reads as a token of its own.
    pub fn scanner_difference(&self) -> Option<TextSize> {
        let sql = self.text;
        // The scanner reads a C string, which ends at the first NUL.
        if sql.contains('\0')
            || self.kind.contains(&SyntaxKind::META_COMMAND)
            || !self.copy_data.is_empty()
        {
            return None;
        }
        let scanned = pgls_query::scan(sql).ok()?;
        let compared = |range: &TextRange| {
            let text = &sql[*range];
            !text.trim().is_empty()
                && text != "$"
                && !text.chars().all(|c| OPERATOR_CHARS.contains(c))
        };
        let scanner = scanned
            .tokens
            .iter()
            .filter_map(|token| {
                Some(TextRange::new(
                    TextSize::from(u32::try_from(token.start).ok()?),
                    TextSize::from(u32::try_from(token.end).ok()?),
                ))
            })
            .filter(compared);
        let mut lexer = (0..self.len())
            .map(|index| {
                let range = self.range(index);
                if self.kind(index) == SyntaxKind::NAMED_PARAM {
                    TextRange::new(range.start() + TextSize::of(':'), range.end())
                } else {
                    range
                }
            })
            .filter(compared);
        for scanner in scanner {
            match lexer.next() {
                Some(lexer) if lexer == scanner => {}
                Some(lexer) => return Some(lexer.start().min(scanner.start())),
                None => return Some(scanner.start()),
            }
        }
        lexer.next().map(|lexer| lexer.start())
    }
}
