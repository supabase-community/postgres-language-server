//! How many suppressions a diagnostic looks at, in a generated 20k-statement file. The budget is
//! that of checkpoint 0 (S8): the suppressions near the diagnostic, plus binary searches.

use pgls_test_utils::scale::Dump;
use pgls_text_size::{TextRange, TextSize};

use crate::tests::TestDiagnostic;
use crate::{LookedAt, Suppressions};

const STATEMENTS: usize = 20_000;

#[test]
fn a_diagnostic_looks_at_the_suppressions_near_it() {
    let dump = Dump::new(STATEMENTS);
    for every in [1000, 100, 10] {
        // A line suppression above every `every`-th statement, and a diagnostic on each one.
        let mut text = String::new();
        let mut statements = Vec::new();
        for (index, statement) in dump.statement_texts().enumerate() {
            if index % every == 0 {
                text.push_str("-- pgls-ignore lint\n");
            }
            let start = TextSize::of(text.as_str());
            text.push_str(statement);
            statements.push(TextRange::at(start, TextSize::of(statement)));
            text.push_str("\n\n");
        }
        let suppressions = Suppressions::from(text.as_str()).with_statements(statements.clone());
        let lines = suppressions.line_suppressions.len();
        assert_eq!(lines, STATEMENTS.div_ceil(every));

        let (suppressed, looked_at) = LookedAt::during(|| {
            statements
                .iter()
                .filter(|&&span| suppressions.is_suppressed(&TestDiagnostic { span }))
                .count()
        });
        assert_eq!(suppressed, lines);
        // Four binary searches over the line suppressions per diagnostic, and the one
        // suppression above its statement.
        let searches = 4 * (lines.ilog2() as usize + 2);
        let budget = STATEMENTS * (searches + 1);
        assert!(
            looked_at <= budget,
            "{lines} line suppressions: {looked_at} looked at for {STATEMENTS} diagnostics, \
             budget {budget}"
        );
    }
}
