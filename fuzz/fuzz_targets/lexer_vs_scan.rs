//! `pgls_lexer` cuts any text Postgres' scanner (`pgls_query::scan`) accepts into the same tokens.
//! Whitespace and operators aren't compared, and named parameters (`:name`, `@name`, `$name`) are
//! compared without their prefix, which the scanner reads as a token of its own.

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
    // Everything but whitespace, operators and the `$` of `$name` is compared.
    let compared = |range: TextRange| {
        let text = &sql[range];
        !text.trim().is_empty() && text != "$" && !text.chars().all(|c| OPERATOR_CHARS.contains(c))
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
        .filter(|&range| compared(range));
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
        .filter(|&range| compared(range));
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
    let text = |range: Option<TextRange>| range.map(|range| &sql[range]);
    panic!(
        "lexer_vs_scan: the lexer's token differs from the scanner's\nlexer: {lexer:?} {:?}\n\
         scanner: {scanner:?} {:?}\ninput: {sql:?}",
        text(lexer),
        text(scanner)
    );
});
