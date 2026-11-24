//! `pgls_lexer` cuts any text Postgres' scanner (`pgls_query::scan`) accepts into the same tokens,
//! as `crates/pgls_postgres_regress/tests/lexer.rs` checks on the regression suite: whitespace and
//! operators aren't compared, and named parameters (`:name`, `@name`, `$name`) are compared without
//! their prefix, which the scanner reads as a token of its own.
//!
//! Two known differences are skipped, until the lexer handles them: string constants continued
//! across a line break (`'a'\n'b'`), which the scanner reads as one token, and `''` in a bit or
//! hex string (`b''''`), which the scanner reads as the end of the string and an empty one.

#![no_main]

use libfuzzer_sys::fuzz_target;
use pgls_lexer::SyntaxKind;
use pgls_text_size::{TextRange, TextSize};

/// The characters of Postgres' operators (`op_chars` in `scan.l`), and `:` of `::` and `:=`.
///
/// https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/scan.l#L367
const OPERATOR_CHARS: &str = "~!@#^&|`?+-*/%<>=:";

fuzz_target!(|data: &[u8]| {
    let Ok(sql) = std::str::from_utf8(data) else {
        return;
    };
    // The scanner reads a C string, which ends at the first NUL.
    if sql.contains('\0') {
        return;
    }
    let Ok(scanned) = pgls_query::scan(sql) else {
        return;
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
        .filter(|&range| Token::compared(sql, range));
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
        .filter(|&range| Token::compared(sql, range));
    let mut pairs = lexer
        .map(Some)
        .chain(std::iter::repeat(None))
        .zip(scanner.map(Some).chain(std::iter::repeat(None)));
    let Some((lexer, scanner)) = pairs.find(|(lexer, scanner)| lexer != scanner || lexer.is_none())
    else {
        return;
    };
    if lexer.is_none() && scanner.is_none() {
        return;
    }
    if let (Some(lexer), Some(scanner)) = (lexer, scanner)
        && Token::known_difference(sql, lexer, scanner)
    {
        return;
    }
    let text = |range: Option<TextRange>| range.map(|range| &sql[range]);
    panic!(
        "lexer_vs_scan: the lexer's token differs from the scanner's\nlexer: {lexer:?} {:?}\n\
         scanner: {scanner:?} {:?}\ninput: {sql:?}",
        text(lexer),
        text(scanner)
    );
});

struct Token;

impl Token {
    /// Whether a token is compared: everything but whitespace, operators and the `$` of `$name`.
    fn compared(sql: &str, range: TextRange) -> bool {
        let text = &sql[range];
        !text.trim().is_empty() && text != "$" && !text.chars().all(|c| OPERATOR_CHARS.contains(c))
    }

    /// Whether the first difference is one of the two known ones, see the module's documentation.
    fn known_difference(sql: &str, lexer: TextRange, scanner: TextRange) -> bool {
        if lexer.start() != scanner.start() {
            return false;
        }
        let (lexer, scanner) = (&sql[lexer], &sql[scanner]);
        let continued = scanner
            .strip_prefix(lexer)
            .is_some_and(|rest| lexer.ends_with('\'') && rest.contains(['\n', '\r']));
        let bit_or_hex = lexer.len() > scanner.len()
            && lexer.starts_with(['b', 'B', 'x', 'X'])
            && lexer.contains("''");
        continued || bit_or_hex
    }
}
