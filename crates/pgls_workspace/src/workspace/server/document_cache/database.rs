//! What the checks against the database found for each statement of a document.
//!
//! This is the interim step of the owner's decision OD5: the language server runs the database
//! checks when a file is opened or saved, and a pull while the user types reuses their results
//! for the statements whose text is unchanged. The statement table (plan item P16) replaces this
//! storage with a column of the statement.

use pgls_diagnostics::{DiagnosticExt, serde::Diagnostic as SDiagnostic};
use pgls_text_size::TextRange;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::workspace::StatementId;

/// What the checks against the database found for one statement.
#[derive(Default)]
pub(crate) struct StatementDatabaseResult {
    /// The diagnostics, with spans relative to the start of the statement.
    pub(crate) diagnostics: Vec<SDiagnostic>,
    /// Whether plpgsql_check checked the statement. It locates the syntax errors of a PL/pgSQL
    /// body, so the syntax error of libpg_query, which spans the whole body, isn't reported.
    pub(crate) plpgsql_checked: bool,
}

impl StatementDatabaseResult {
    fn is_empty(&self) -> bool {
        self.diagnostics.is_empty() && !self.plpgsql_checked
    }
}

/// What the last run of the database checks found for the top-level statements of a document,
/// by statement text: the results of the occurrences of the text, in document order. Statements
/// without a result aren't kept, so it holds at most one entry per statement of the document.
#[derive(Default)]
pub(crate) struct DatabaseResults {
    statements: FxHashMap<StatementId, Vec<StatementDatabaseResult>>,
}

/// The diagnostics of [`DatabaseResults::reuse`], and the ranges of the statements that
/// plpgsql_check checked.
pub(crate) type ReusedDatabaseResults = (Vec<SDiagnostic>, Vec<TextRange>);

impl DatabaseResults {
    /// The results of the checked top-level statements, `(statement, range, result)` in document
    /// order. Each one has its own diagnostics, also when the document repeats a statement's text.
    pub(crate) fn new(
        checked: impl IntoIterator<Item = (StatementId, TextRange, StatementDatabaseResult)>,
    ) -> Self {
        let checked: Vec<_> = checked.into_iter().collect();
        // The texts with a result keep the results of all of their occurrences, so that
        // `reuse` finds an occurrence's result by its position.
        let with_results: FxHashSet<StatementId> = checked
            .iter()
            .filter(|(_, _, result)| !result.is_empty())
            .map(|(id, _, _)| id.clone())
            .collect();
        let mut statements: FxHashMap<StatementId, Vec<StatementDatabaseResult>> =
            FxHashMap::default();
        for (id, range, result) in checked {
            if !with_results.contains(&id) {
                continue;
            }
            debug_assert!(
                result.diagnostics.iter().all(|diagnostic| diagnostic
                    .span()
                    .is_none_or(|span| span.end() <= range.len())),
                "the spans are relative to the statement"
            );
            statements.entry(id).or_default().push(result);
        }
        Self { statements }
    }

    /// The results for the top-level statements `statements`, `(text, range)` in document order,
    /// with the spans moved to where each statement is now. A text whose number of occurrences
    /// changed has none, since its occurrences can't be told apart.
    pub(crate) fn reuse<'a>(
        &self,
        statements: impl IntoIterator<Item = (&'a str, TextRange)>,
    ) -> ReusedDatabaseResults {
        let mut diagnostics = Vec::new();
        let mut plpgsql_checked = Vec::new();
        if self.statements.is_empty() {
            return (diagnostics, plpgsql_checked);
        }

        // The statements with results, and how often each text occurs in the document.
        let mut found: Vec<(&StatementId, &[StatementDatabaseResult], usize, TextRange)> =
            Vec::new();
        let mut occurrences: FxHashMap<&StatementId, usize> = FxHashMap::default();
        for (text, range) in statements {
            if let Some((id, results)) = self.statements.get_key_value(&StatementId::new(text)) {
                let occurrence = occurrences.entry(id).or_default();
                found.push((id, results.as_slice(), *occurrence, range));
                *occurrence += 1;
            }
        }

        for (id, results, occurrence, range) in found {
            if occurrences.get(id) != Some(&results.len()) {
                continue;
            }
            let Some(result) = results.get(occurrence) else {
                continue;
            };
            if result.plpgsql_checked {
                plpgsql_checked.push(range);
            }
            diagnostics.extend(result.diagnostics.iter().map(|diagnostic| {
                let span = diagnostic.span().map(|span| span + range.start());
                debug_assert!(span.is_none_or(|span| range.contains_range(span)));
                SDiagnostic::new(diagnostic.clone().with_file_span(span))
            }));
        }
        (diagnostics, plpgsql_checked)
    }

    /// Drops the results of the statements with the text `text`, which the document no longer
    /// has.
    pub(crate) fn remove(&mut self, text: &StatementId) {
        self.statements.remove(text);
    }

    pub(crate) fn shrink(&mut self) {
        self.statements.shrink_to_fit();
    }
}

#[cfg(test)]
mod tests {
    use pgls_diagnostics::{Diagnostic, DiagnosticExt, Error, serde::Diagnostic as SDiagnostic};
    use pgls_text_size::TextRange;

    use super::{DatabaseResults, StatementDatabaseResult};
    use crate::workspace::StatementId;

    #[derive(Debug, Diagnostic)]
    #[diagnostic(category = "plpgsql_check", message = "not found")]
    struct NotFound;

    impl StatementDatabaseResult {
        /// A statement that plpgsql_check checked, with a diagnostic at `span` of the statement.
        fn checked_with_diagnostic(span: TextRange) -> Self {
            Self {
                diagnostics: vec![SDiagnostic::new(
                    Error::from(NotFound).with_file_span(span),
                )],
                plpgsql_checked: true,
            }
        }
    }

    /// The results move with their statement, and a statement whose text changed has none.
    #[test]
    fn reuses_the_results_of_unchanged_statements() {
        let results = DatabaseResults::new([
            (
                StatementId::new("select 1;"),
                TextRange::new(0.into(), 9.into()),
                StatementDatabaseResult::default(),
            ),
            (StatementId::new("select a;"), TextRange::new(10.into(), 19.into()), StatementDatabaseResult::checked_with_diagnostic(TextRange::new(7.into(), 8.into()))),
            (StatementId::new("select b;"), TextRange::new(20.into(), 29.into()), StatementDatabaseResult::checked_with_diagnostic(TextRange::new(7.into(), 8.into()))),
        ]);

        // A statement was added before them, and `select b;` changed.
        let reused = results.reuse([
            ("select 0;", TextRange::new(0.into(), 9.into())),
            ("select 1;", TextRange::new(10.into(), 19.into())),
            ("select a;", TextRange::new(20.into(), 29.into())),
            ("select c;", TextRange::new(30.into(), 39.into())),
        ]);

        assert_eq!(reused.0.iter().map(SDiagnostic::span).collect::<Vec<_>>(), [Some(TextRange::new(27.into(), 28.into()))]);
        assert_eq!(reused.1, [TextRange::new(20.into(), 29.into())]);
    }

    /// Each occurrence of a repeated statement keeps its own results, as long as the number of
    /// occurrences is unchanged.
    #[test]
    fn reuses_the_results_of_each_occurrence() {
        let results = DatabaseResults::new([
            (
                StatementId::new("select a;"),
                TextRange::new(0.into(), 9.into()),
                StatementDatabaseResult::default(),
            ),
            (StatementId::new("select a;"), TextRange::new(10.into(), 19.into()), StatementDatabaseResult::checked_with_diagnostic(TextRange::new(7.into(), 8.into()))),
        ]);

        let moved = results.reuse([("select a;", TextRange::new(5.into(), 14.into())), ("select a;", TextRange::new(20.into(), 29.into()))]);
        assert_eq!(moved.0.iter().map(SDiagnostic::span).collect::<Vec<_>>(), [Some(TextRange::new(27.into(), 28.into()))]);

        let removed = results.reuse([("select a;", TextRange::new(0.into(), 9.into()))]);
        assert_eq!(removed.0.iter().map(SDiagnostic::span).collect::<Vec<_>>(), []);
        assert!(removed.1.is_empty());
    }
}
