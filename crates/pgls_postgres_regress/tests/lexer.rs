//! Checks that `pgls_lexer` and Postgres' scanner (`pgls_query::scan`) cut every statement of
//! Postgres' regression suite, of every supported version, at the same places. The differences
//! are snapshotted per version, so a new one fails the test and a fixed one shows up as a diff.
//!
//! Where the two disagree, the splitter and every feature that reads the lexer's tokens see a
//! different statement than Postgres does, e.g. text after a lone `\r` in a `--` comment.
//!
//! Whitespace isn't compared, since the scanner doesn't return it, and neither are operators: the
//! lexer has a token per character of an operator, the scanner one per operator. The lexer reads
//! psql variables and `@name` as named parameters (`:name`, `:'name'`, `@name`), where the scanner
//! reads the prefix as an operator, so a named parameter is compared without its prefix.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "a test fails on unexpected data; clippy exempts `#[test]` functions, not their helpers"
)]

use pgls_lexer::SyntaxKind;
use pgls_postgres_regress::{File, Statement};
use pgls_text_size::{TextRange, TextSize};

/// The characters of Postgres' operators (`op_chars` in `scan.l`), and `:` of `::` and `:=`.
///
/// https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/scan.l#L367
const OPERATOR_CHARS: &str = "~!@#^&|`?+-*/%<>=:";

/// How many characters of a token the report shows.
const REPORTED_CHARS_MAX: usize = 40;

#[test]
fn lexer_agrees_with_scanner() {
    for version in pgls_postgres_regress::versions() {
        let (mut statements, mut unscanned) = (0, 0);
        let mut entries = Vec::new();
        for file in version.files() {
            for statement in &file.statements {
                statements += 1;
                match Difference::of(&file, statement) {
                    Ok(None) => {}
                    Ok(Some(difference)) => entries.push(difference.entry(&file)),
                    Err(()) => unscanned += 1,
                }
            }
        }
        let mut report = format!(
            "{}: {statements} statements, {unscanned} the scanner rejects, {} differences\n",
            version.tag,
            entries.len()
        );
        for entry in entries {
            report.push('\n');
            report.push_str(&entry);
        }
        insta::assert_snapshot!(format!("lexer_vs_scanner_{}", version.major), report);
    }
}

/// The first token where the lexer and the scanner differ, in [`File::source`]. `None` on a side
/// that has no more tokens.
struct Difference {
    lexer: Option<TextRange>,
    scanner: Option<TextRange>,
}

impl Difference {
    /// The first difference in a statement, or `Err` if the scanner rejects it, e.g. an
    /// unterminated string.
    fn of(file: &File, statement: &Statement) -> Result<Option<Difference>, ()> {
        let sql = file.sql(statement);
        let scanned = pgls_query::scan(sql).map_err(|_| ())?;
        let scanner = scanned
            .tokens
            .iter()
            .map(|token| {
                TextRange::new(
                    TextSize::from(u32::try_from(token.start).unwrap()),
                    TextSize::from(u32::try_from(token.end).unwrap()),
                )
            })
            .filter(|&range| Self::compared(&sql[range]));
        let lexed = pgls_lexer::lex(sql);
        let lexer = (0..lexed.len())
            .map(|index| {
                let range = lexed.range(index);
                if lexed.kind(index) == SyntaxKind::NAMED_PARAM {
                    TextRange::new(range.start() + TextSize::of(':'), range.end())
                } else {
                    range
                }
            })
            .filter(|&range| Self::compared(&sql[range]));
        let mut pairs = lexer
            .map(Some)
            .chain(std::iter::repeat(None))
            .zip(scanner.map(Some).chain(std::iter::repeat(None)));
        let (lexer, scanner) = pairs
            .find(|(lexer, scanner)| lexer != scanner || lexer.is_none())
            .unwrap();
        let start = statement.range.start();
        Ok((lexer.is_some() || scanner.is_some()).then(|| Difference {
            lexer: lexer.map(|range| range + start),
            scanner: scanner.map(|range| range + start),
        }))
    }

    /// Whether a token's text is compared: everything but whitespace and operators.
    fn compared(text: &str) -> bool {
        !text.trim().is_empty() && !text.chars().all(|c| OPERATOR_CHARS.contains(c))
    }

    fn entry(&self, file: &File) -> String {
        let start = [self.lexer, self.scanner]
            .into_iter()
            .flatten()
            .map(|range| range.start())
            .min()
            .unwrap();
        let (line, col) = file.line_col_at(start);
        let mut entry = format!("{}.sql:{line}:{col}:\n", file.name);
        for (side, range) in [("lexer", self.lexer), ("scanner", self.scanner)] {
            let token = match range {
                Some(range) => {
                    let (line, col) = file.line_col_at(range.start());
                    format!("{line}:{col} {:?}", Self::excerpt(&file.source[range]))
                }
                None => "end of statement".to_owned(),
            };
            entry.push_str(&format!("  {side:<8} {token}\n"));
        }
        entry
    }

    /// The token, shortened to [`REPORTED_CHARS_MAX`] characters.
    fn excerpt(token: &str) -> String {
        if token.chars().count() <= REPORTED_CHARS_MAX {
            token.to_owned()
        } else {
            format!(
                "{}…",
                token.chars().take(REPORTED_CHARS_MAX).collect::<String>()
            )
        }
    }
}
