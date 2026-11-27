//! The formatter on any text that parses as one statement: `format_statement` doesn't panic,
//! doesn't refuse because its output doesn't parse back to the same AST, and its output formats to
//! itself (`FormatResult::formats_to_itself`). The same oracle as the corpus test.

#![no_main]

use libfuzzer_sys::fuzz_target;
use pgls_pretty_print::{FormatConfig, FormatError, FormatStatementParams, format_statement};

/// Longer inputs mostly cost time; a statement's layout doesn't need them.
const INPUT_LEN_MAX: usize = 2000;

fuzz_target!(|data: &[u8]| {
    let Ok(sql) = std::str::from_utf8(data) else {
        return;
    };
    if sql.len() > INPUT_LEN_MAX {
        return;
    }
    let Some(ast) = pgls_query::parse(sql).ok().and_then(|x| x.into_root()) else {
        return;
    };
    let config = FormatConfig {
        line_width: 80,
        ..Default::default()
    };
    match format_statement(FormatStatementParams::new(&ast, sql, &config)) {
        Ok(result) => assert!(
            result.formats_to_itself(&config),
            "formatter_roundtrip: formatting isn't idempotent\n{sql}\n---\n{}",
            result.formatted
        ),
        Err(e @ (FormatError::ParseError { .. } | FormatError::BetaUnsupported { .. })) => {
            panic!("formatter_roundtrip: the output doesn't round-trip: {e}\n{sql}")
        }
        Err(_) => {}
    }
});
