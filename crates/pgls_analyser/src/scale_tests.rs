//! How many statements a lookup of an analysis compares, for each statement of a generated
//! 20k-statement file. The budget is that of checkpoint 0 (S8): a binary search.

use pgls_test_utils::scale::Dump;
use pgls_text_size::{TextRange, TextSize};

use crate::{AnalysisResult, LookupSteps, StatementAnalysis};

const STATEMENTS: usize = 20_000;

#[test]
fn a_lookup_by_range_is_a_binary_search() {
    let dump = Dump::new(STATEMENTS);
    let mut start = TextSize::new(0);
    let ranges: Vec<TextRange> = dump
        .statement_texts()
        .map(|statement| {
            let range = TextRange::at(start, TextSize::of(statement));
            start = range.end() + TextSize::of("\n\n");
            range
        })
        .collect();
    let analysis = AnalysisResult {
        statements: ranges
            .iter()
            .map(|&range| StatementAnalysis {
                range,
                database_only: false,
                reported_database_error: false,
                search_path: Vec::new(),
            })
            .collect(),
        ..AnalysisResult::default()
    };

    let (found, steps) = LookupSteps::during(|| {
        ranges
            .iter()
            .filter(|&&range| analysis.statement(range).is_some())
            .count()
    });
    assert_eq!(found, STATEMENTS);
    let budget = STATEMENTS * (STATEMENTS.ilog2() as usize + 2);
    assert!(
        steps <= budget,
        "{steps} statements compared for {STATEMENTS} lookups, budget {budget}"
    );
}
