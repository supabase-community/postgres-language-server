//! The lexer compared with Postgres' scanner, for the lexer's tests, the corpus check and the fuzz
//! target `lexer_vs_scan`.

use pgls_text_size::{TextRange, TextSize};
use pgls_tokenizer::CharClass;

use crate::{Lexed, SyntaxKind};

impl Lexed<'_> {
    /// Where the tokens first differ from those of Postgres' scanner (`pgls_query::scan`), or
    /// `None` if they agree, the scanner rejects the text, or the text has psql meta-commands,
    /// psql variables (`:name`, `:'name'`, ...) or COPY data, which the scanner reads as SQL.
    /// Whitespace isn't compared. The other named parameters (`@name`, `$name`) are compared as
    /// their prefix and their name, which the scanner reads as two tokens. A text with an `@name`
    /// right after an operator isn't compared either: the scanner reads the `@` into the
    /// operator.
    pub fn scanner_difference(&self) -> Option<TextSize> {
        let sql = self.text;
        let at_param_after_operator = |index| {
            self.text(index).starts_with('@')
                && sql
                    .get(..usize::from(self.range(index).start()))
                    .is_some_and(|before| before.ends_with(|c| CharClass::OpChar.contains(c)))
        };
        // The scanner reads a C string, which ends at the first NUL.
        if sql.contains('\0')
            || self.kind.contains(&SyntaxKind::META_COMMAND)
            || !self.copy_data.is_empty()
            || (0..self.len()).any(|index| {
                self.kind(index) == SyntaxKind::NAMED_PARAM
                    && (self.text(index).starts_with(':') || at_param_after_operator(index))
            })
        {
            return None;
        }
        let scanned = pgls_query::scan(sql).ok()?;
        let compared = |range: &TextRange| !sql[*range].trim().is_empty();
        let scanner = scanned
            .tokens
            .iter()
            .filter_map(|token| token.range())
            .filter(compared);
        let mut lexer = (0..self.len())
            .flat_map(|index| {
                let range = self.range(index);
                if self.kind(index) == SyntaxKind::NAMED_PARAM {
                    let name = range.start() + TextSize::of('@');
                    [
                        Some(TextRange::new(range.start(), name)),
                        Some(TextRange::new(name, range.end())),
                    ]
                } else {
                    [Some(range), None]
                }
            })
            .flatten()
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
