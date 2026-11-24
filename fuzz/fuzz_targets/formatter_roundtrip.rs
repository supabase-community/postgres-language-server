//! The formatter on any text that parses as one statement: `format_statement` doesn't panic, and
//! the output without comments parses back to the same AST and formats to itself. The same
//! oracle as `crates/pgls_pretty_print/tests/postgres_regress.rs`, on inputs the regression suite
//! doesn't have.

#![no_main]

use libfuzzer_sys::fuzz_target;
use pgls_pretty_print::{
    FormatConfig, format_statement, format_statement_unchecked, normalize_ast,
};

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

    // The editor's path checks its own output and may refuse, but never panics.
    let _ = format_statement(&ast, sql, &config);

    let Ok(output) = format_statement_unchecked(&ast, &config) else {
        return;
    };
    let reparsed = match pgls_query::parse(&output).map(|x| x.into_root()) {
        Ok(Some(reparsed)) => reparsed,
        Ok(None) => panic!("formatter_roundtrip: output isn't one statement\n{sql}\n---\n{output}"),
        Err(e) => panic!("formatter_roundtrip: output doesn't parse: {e}\n{sql}\n---\n{output}"),
    };
    let (mut expected, mut actual) = (ast.clone(), reparsed.clone());
    normalize_ast(&mut expected);
    normalize_ast(&mut actual);
    assert!(
        expected == actual,
        "formatter_roundtrip: output has a different AST\n{sql}\n---\n{output}"
    );
    let again = format_statement_unchecked(&reparsed, &config);
    assert!(
        again.as_ref().is_ok_and(|again| *again == output),
        "formatter_roundtrip: formatting isn't idempotent\n{sql}\n---\n{output}\n---\n{again:?}"
    );
});
