//! The lexer and the splitter on any text: the lexer's tokens are contiguous and give back the
//! input, and the statements are ordered, not empty, and in bounds and on char boundaries.

#![no_main]

use libfuzzer_sys::fuzz_target;
use pgls_text_size::TextSize;

fuzz_target!(|data: &[u8]| {
    let Ok(sql) = std::str::from_utf8(data) else {
        return;
    };

    let lexed = pgls_lexer::lex(sql);
    let mut end = TextSize::from(0);
    for index in 0..lexed.len() {
        let range = lexed.range(index);
        assert!(
            range.start() == end
                && sql
                    .get(usize::from(range.start())..usize::from(range.end()))
                    .is_some(),
            "splitter: lexer tokens not contiguous or off a char boundary\ntoken {index} at \
             {range:?}\ninput: {sql:?}"
        );
        end = range.end();
    }
    assert_eq!(
        end,
        TextSize::of(sql),
        "splitter: lexer tokens don't cover the input\ninput: {sql:?}"
    );

    let split = pgls_statement_splitter::split(sql);
    let mut end = TextSize::from(0);
    for &range in &split.ranges {
        assert!(
            !range.is_empty()
                && range.start() >= end
                && sql
                    .get(usize::from(range.start())..usize::from(range.end()))
                    .is_some(),
            "splitter: statement empty, overlapping, out of bounds or off a char boundary\nrange \
             {range:?} of {:?}\ninput: {sql:?}",
            split.ranges
        );
        end = range.end();
    }
});
