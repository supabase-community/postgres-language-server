//! `pgls_lexer` cuts any text Postgres' scanner (`pgls_query::scan`) accepts into the same tokens,
//! as `Lexed::scanner_difference` compares them.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(sql) = std::str::from_utf8(data) else {
        return;
    };
    let lexed = pgls_lexer::lex(sql);
    if let Some(offset) = lexed.scanner_difference() {
        panic!(
            "lexer_vs_scan: the lexer's tokens differ from the scanner's at {offset:?}\ninput: {sql:?}"
        );
    }
});
